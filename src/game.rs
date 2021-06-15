use ash::version::DeviceV1_0;
use ash::vk;
use std::cell::RefCell;
use std::ffi::CString;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::atomic::AtomicI32;
use std::sync::atomic::AtomicPtr;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::Arc;
use wyzoid::high::job::JobTimingsBuilder;
use wyzoid::low::vkcmd;
use wyzoid::low::vkdescriptor;
use wyzoid::low::vkfence;
use wyzoid::low::vkmem;
use wyzoid::low::vkpipeline;
use wyzoid::low::vkshader;
use wyzoid::low::vkstate::VulkanState;

#[derive(PartialEq, Clone, Copy)]
#[repr(C)]
pub enum Concept {
    Soil = 0,
    Sunflower = 1,
    Rose = 2,
    Dogwood = 3,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct CellState {
    pub concept: Concept,
    pub blood: i32,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Mutation {
    pub x: u32,
    pub y: u32,
    pub concept: Concept,
}

#[repr(C)]
struct ShaderParams {
    world_width: u32,
    flip: i32,
    mutations_size: u32,
    mutations: [Mutation; 100],
}

pub struct Runner {
    vulkan: Rc<VulkanState>,
    memory: vkmem::VkMem,
    world_width: u32,
    timing: JobTimingsBuilder,
    fence: vkfence::VkFence,
    cmd_pool: vkcmd::VkCmdPool,
    flip: i32,
    left_data: *mut CellState,
    right_data: *mut CellState,
    param_data: *mut ShaderParams,
    paused: bool,

    mutations: Vec<Mutation>,
    pub game_state: Arc<GameState>,
}

pub struct GameState {
    pub game_data: AtomicPtr<CellState>,
    pub game_size: AtomicI32,
}

impl Runner {
    pub fn new(world_width: u32) -> Runner {
        let mut state = vec![
            CellState {
                concept: Concept::Soil,
                blood: 0
            };
            (world_width * world_width) as usize
        ];

        // let's make a nice default pattern
        for i in 1..(world_width - 1) {
            state[(1 + i * world_width) as usize].concept = Concept::Sunflower;
            state[((world_width - 2) + i * world_width) as usize].concept = Concept::Sunflower;
        }
        for j in 2..(world_width - 2) {
            state[(world_width + j) as usize].concept = Concept::Sunflower;
            state[((world_width - 2) * world_width + j) as usize].concept = Concept::Sunflower;
        }

        // Memory init.
        let mut timing: JobTimingsBuilder = JobTimingsBuilder::new();
        timing = timing.start_upload();
        let shader_path = PathBuf::from("target/game.spv");
        let vulkan = Rc::new(wyzoid::low::vkstate::init_vulkan());
        let buffer_size: u64 = (state.len() * std::mem::size_of::<CellState>()) as u64;

        // Make two buffers and bind them to GPU memory
        let mut left_buffer = vkmem::VkBuffer::new(vulkan.clone(), buffer_size);
        let mut right_buffer = vkmem::VkBuffer::new(vulkan.clone(), buffer_size);
        let mut param_buffer =
            vkmem::VkBuffer::new(vulkan.clone(), std::mem::size_of::<ShaderParams>() as u64);
        let (mem_size, offsets) = vkmem::compute_non_overlapping_buffer_alignment(&vec![
            &left_buffer,
            &right_buffer,
            &param_buffer,
        ]);
        let memory = vkmem::VkMem::find_mem(vulkan.clone(), mem_size)
            .expect("[ERR] Could not find a memory type fitting our need.");

        left_buffer.bind(memory.mem, offsets[0]);
        right_buffer.bind(memory.mem, offsets[1]);
        param_buffer.bind(memory.mem, offsets[2]);

        // Try mapping all three buffers here, and leave them mapped
        let left_data: *mut CellState = unsafe {
            vulkan
                .device
                .map_memory(
                    memory.mem,
                    left_buffer.offset,
                    left_buffer.size,
                    vk::MemoryMapFlags::empty(),
                )
                .expect("[ERR] Could not map memory.") as *mut CellState
        };
        unsafe {
            std::ptr::copy_nonoverlapping(state.as_ptr(), left_data, state.len());
        }
        let right_data: *mut CellState = unsafe {
            vulkan
                .device
                .map_memory(
                    memory.mem,
                    right_buffer.offset,
                    right_buffer.size,
                    vk::MemoryMapFlags::empty(),
                )
                .expect("[ERR] Could not map memory.") as *mut CellState
        };
        let param_data: *mut ShaderParams = unsafe {
            vulkan
                .device
                .map_memory(
                    memory.mem,
                    param_buffer.offset,
                    param_buffer.size,
                    vk::MemoryMapFlags::empty(),
                )
                .expect("[ERR] Could not map memory.") as *mut ShaderParams
        };
        unsafe {
            std::ptr::copy_nonoverlapping(
                vec![ShaderParams {
                    world_width: world_width,
                    flip: 0,
                    mutations_size: 0,
                    mutations: [Mutation {
                        x: 0,
                        y: 0,
                        concept: Concept::Soil,
                    }; 100],
                }]
                .as_ptr(),
                param_data,
                1,
            );
        }
        timing = timing.stop_upload();

        // Shaders
        timing = timing.start_shader();

        // Create the shader, and bind to its layouts.
        let shader = Rc::new(RefCell::new(vkshader::VkShader::new(
            vulkan.clone(),
            &shader_path,
            CString::new("main").unwrap(),
        )));
        shader.borrow_mut().add_layout_binding(
            0,
            1,
            vk::DescriptorType::STORAGE_BUFFER,
            vk::ShaderStageFlags::COMPUTE,
        );
        shader.borrow_mut().add_layout_binding(
            1,
            1,
            vk::DescriptorType::STORAGE_BUFFER,
            vk::ShaderStageFlags::COMPUTE,
        );
        shader.borrow_mut().add_layout_binding(
            2,
            1,
            vk::DescriptorType::STORAGE_BUFFER,
            vk::ShaderStageFlags::COMPUTE,
        );
        shader.borrow_mut().create_pipeline_layout();
        let pipeline_layout = shader.borrow().pipeline.unwrap();
        let compute_pipeline = vkpipeline::VkComputePipeline::new(vulkan.clone(), &shader.borrow());
        let mut shader_descriptor = vkdescriptor::VkDescriptor::new(vulkan.clone(), shader.clone());
        let mut write_descriptor = vkdescriptor::VkWriteDescriptor::new(vulkan.clone());

        shader_descriptor.add_pool_size(3, vk::DescriptorType::STORAGE_BUFFER);
        shader_descriptor.create_pool(1);
        shader_descriptor.create_set();

        let desc_set: vk::DescriptorSet = *shader_descriptor.get_first_set().unwrap();
        let mut buffers_nfos: Vec<Vec<vk::DescriptorBufferInfo>> = Vec::new();

        // Add write descriptors for our two buffers
        write_descriptor.add_buffer(left_buffer.buffer, 0, left_buffer.size);
        buffers_nfos.push(vec![write_descriptor.buffer_descriptors[0]]);
        write_descriptor.add_write_descriptors(
            desc_set,
            vk::DescriptorType::STORAGE_BUFFER,
            &buffers_nfos[0],
            0,
            0,
        );
        write_descriptor.add_buffer(right_buffer.buffer, 0, right_buffer.size);
        buffers_nfos.push(vec![write_descriptor.buffer_descriptors[1]]);
        write_descriptor.add_write_descriptors(
            desc_set,
            vk::DescriptorType::STORAGE_BUFFER,
            &buffers_nfos[1],
            1,
            0,
        );
        write_descriptor.add_buffer(param_buffer.buffer, 0, param_buffer.size);
        buffers_nfos.push(vec![write_descriptor.buffer_descriptors[2]]);
        write_descriptor.add_write_descriptors(
            desc_set,
            vk::DescriptorType::STORAGE_BUFFER,
            &buffers_nfos[2],
            2,
            0,
        );

        write_descriptor.update_descriptors_sets();

        timing = timing.stop_shader();

        // Command buffers
        timing = timing.start_cmd();
        let mut cmd_pool = vkcmd::VkCmdPool::new(vulkan.clone());

        let cmd_buffer = cmd_pool.create_cmd_buffer(vk::CommandBufferLevel::PRIMARY);
        cmd_pool.begin_cmd(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT, cmd_buffer);
        cmd_pool.bind_pipeline(
            compute_pipeline.pipeline,
            vk::PipelineBindPoint::COMPUTE,
            cmd_buffer,
        );
        cmd_pool.bind_descriptor(
            pipeline_layout,
            vk::PipelineBindPoint::COMPUTE,
            &shader_descriptor.set,
            cmd_buffer,
        );

        cmd_pool.dispatch((state.len() / 25) as u32, 1, 1, cmd_buffer);

        // Memory barrier
        let mut buffer_barrier: Vec<vk::BufferMemoryBarrier> = Vec::new();
        buffer_barrier.push(
            vk::BufferMemoryBarrier::builder()
                .src_access_mask(vk::AccessFlags::SHADER_WRITE)
                .dst_access_mask(vk::AccessFlags::SHADER_READ)
                .buffer(left_buffer.buffer)
                .size(vk::WHOLE_SIZE)
                .build(),
        );
        buffer_barrier.push(
            vk::BufferMemoryBarrier::builder()
                .src_access_mask(vk::AccessFlags::SHADER_WRITE)
                .dst_access_mask(vk::AccessFlags::SHADER_READ)
                .buffer(right_buffer.buffer)
                .size(vk::WHOLE_SIZE)
                .build(),
        );
        buffer_barrier.push(
            vk::BufferMemoryBarrier::builder()
                .src_access_mask(vk::AccessFlags::SHADER_WRITE)
                .dst_access_mask(vk::AccessFlags::SHADER_READ)
                .buffer(param_buffer.buffer)
                .size(vk::WHOLE_SIZE)
                .build(),
        );

        unsafe {
            vulkan.device.cmd_pipeline_barrier(
                cmd_pool.cmd_buffers[cmd_buffer],
                vk::PipelineStageFlags::COMPUTE_SHADER,
                vk::PipelineStageFlags::COMPUTE_SHADER,
                vk::DependencyFlags::empty(),
                &[],
                &buffer_barrier,
                &[],
            );
        }

        cmd_pool.end_cmd(cmd_buffer);
        timing = timing.stop_cmd();

        // Execution
        let fence = vkfence::VkFence::new(vulkan.clone(), false);
        return Runner {
            vulkan,
            memory,
            game_state: Arc::new(GameState {
                game_data: AtomicPtr::new(left_data),
                game_size: AtomicI32::new((world_width * world_width) as i32),
            }),
            world_width,
            timing,
            fence,
            cmd_pool,
            flip: 0,
            left_data,
            right_data,
            param_data,
            paused: false,
            mutations: vec![],
        };
    }

    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    pub fn mutate(&mut self, mutation: Mutation) {
        self.mutations.insert(0, mutation);
    }

    pub fn execute(&mut self) {
        if self.paused {
            return;
        };
        let queued_mutations = self
            .mutations
            .splice(0..(std::cmp::min(self.mutations.len(), 100)), vec![]);
        let mut m_array = [Mutation {
            x: 0,
            y: 0,
            concept: Concept::Soil,
        }; 100];
        let m_count = queued_mutations.len() as u32;
        for (i, m) in queued_mutations.enumerate() {
            m_array[i] = m;
        }

        unsafe {
            std::ptr::copy_nonoverlapping(
                vec![ShaderParams {
                    world_width: self.world_width,
                    flip: self.flip,
                    mutations_size: m_count,
                    mutations: m_array,
                }]
                .as_ptr(),
                self.param_data,
                1,
            );
        }

        self.timing = self.timing.start_execution();
        let queue = unsafe {
            self.vulkan
                .device
                .get_device_queue(self.vulkan.queue_family_index, 0)
        };
        self.cmd_pool.submit(queue, Some(self.fence.fence));

        while self.fence.status() != vkfence::FenceStates::SIGNALED {
            self.fence.wait(1 * 1000 * 1000 * 1000);
        }
        self.fence.reset();
        self.timing = self.timing.stop_execution();
        self.flip = if self.flip == 0 { 1 } else { 0 };

        self.game_state.game_data.store(
            if self.flip == 0 {
                self.left_data
            } else {
                self.right_data
            },
            Relaxed,
        );
    }
}
