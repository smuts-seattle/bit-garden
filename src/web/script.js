httpRequest = new XMLHttpRequest();

httpRequest.responseType = "arraybuffer";
httpRequest.onreadystatechange = function(){
  if (httpRequest.readyState === XMLHttpRequest.DONE) {
    const arr = new Int32Array(httpRequest.response);
    console.log(arr);
    var cells = [];
    for (var i = 0; i < arr.length; i += 2)
    {
      cells = cells.concat({concept: arr[i], blood: arr[i+1]})
    }

    console.log(cells);
  }
};
httpRequest.origin = 'localhost';
httpRequest.open('GET', 'http://127.0.0.1:8000', true);
httpRequest.send();