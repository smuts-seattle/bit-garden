const colors = ['#000', '#444', '#777', '#999', '#FFF']
const cell_size = 24;//px

setInterval(function() {
  httpRequest = new XMLHttpRequest();

  httpRequest.responseType = "arraybuffer";
  httpRequest.onreadystatechange = function(){
    if (httpRequest.readyState === XMLHttpRequest.DONE) {
      const arr = new Int32Array(httpRequest.response);
      var cells = [];
      for (var i = 0; i < arr.length; i += 2)
      {
        cells = cells.concat({concept: arr[i], blood: arr[i+1]})
      }
  
      const width = Math.sqrt(cells.length);
      var canvas = document.getElementById('game-area');
      canvas.width = canvas.height = width * cell_size;
      var ctx = canvas.getContext("2d");
  
      for (var i = 0; i < cells.length; i++)
      {
        ctx.fillStyle = colors[cells[i].concept]; 
        ctx.fillRect((i % width) * cell_size, Math.floor(i / width) * cell_size, cell_size, cell_size)
      }
    }
  };
  httpRequest.open('GET', 'http://127.0.0.1:8000', true);
  httpRequest.send();
}, 1000);
