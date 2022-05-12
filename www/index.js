import * as wasm from "automaton-engine";
import { memory } from "automaton-engine/automaton_engine_bg";

const CELL_SIZE = 5; // px
const GRID_COLOR = "#CCCCCC";
const DEAD_COLOR = "#FFFFFF";
const ALIVE_COLOR = "#000000";

// const universe = Universe.new();
// const width = universe.get_width();
// const height = universe.get_height();

wasm.init();
const width = wasm.get_width();
const height = wasm.get_height();

const can = document.getElementById("automaton-canvas");
can.height = (CELL_SIZE + 1) * height + 1;
can.width = (CELL_SIZE + 1) * width + 1;

const ctx = can.getContext('2d');

var animationId;
var tick_count = 9;

class FPS {

  constructor() {
    this.displayElement = document.getElementById("fps");
    this.lastCallTime = performance.now();
    this.frames = [];
  }

  tick() {
    const now = window.performance.now();
    const delta = now - this.lastCallTime;
    this.lastCallTime = now;

    const tpf = 1000 / delta;

    this.frames.push(tpf);

    if(this.frames.length > 100) {
      this.frames.shift();
    }

    var mean = 0;
    var max = -Infinity;
    var min = Infinity;
    for(const x of this.frames) {
      mean += x;
      max = Math.max(max, x);
      min = Math.min(min, x);
    }

    mean /= this.frames.length;


    this.displayElement.textContent = `
Frames per Second:
         latest = ${Math.round(tpf)}
avg of last 100 = ${Math.round(mean)}
min of last 100 = ${Math.round(min)}
max of last 100 = ${Math.round(max)}
`.trim();
  }

}

var fpTimer = new FPS();

const renderLoop = () => {

  fpTimer.tick();

  for(let i = 0; i < tick_count; i++) {
    wasm.tick_life();
  }

  drawGrid();
  drawCells();

  animationId = requestAnimationFrame(renderLoop);
};

const drawGrid = () => {
  ctx.beginPath();
  ctx.strokeStyle = GRID_COLOR;

  //Vertical lines
  for(let i = 0; i <= width; i++) {
    ctx.moveTo(i * (CELL_SIZE + 1) + 1, 0);
    ctx.lineTo(i * (CELL_SIZE + 1) + 1, (CELL_SIZE + 1) * height + 1);
  }

  //Horizontal lines
  for (let i = 0; i <= height; i++) {
    ctx.moveTo(0, i * (CELL_SIZE + 1) + 1);
    ctx.lineTo((CELL_SIZE + 1) * width + 1, i * (CELL_SIZE + 1) + 1);
  }

  ctx.stroke();

};

const drawCells = () => {

  // const cellPtr = universe.get_pointer();
  const cellPtr = wasm.get_pointer();
  const cells = new Uint8Array(memory.buffer, cellPtr, width * height);

  ctx.beginPath();

  ctx.fillStyle = DEAD_COLOR;

  drawInLoop(0, cells);

  ctx.fillStyle = ALIVE_COLOR;

  drawInLoop(1, cells);

  ctx.stroke();

};

const bitValue = (n, arr) => {
  const byte = Math.floor(n / 8);
  //this works because wasm is little endian... super important
  const mask = 1 << (n % 8);
  return (arr[byte] & mask) === mask;
};

const drawInLoop = (t, arr) => {
  for (let row = 0; row < height; row++) {
    for (let col = 0; col < width; col++) {
      const idx = row * width + col;

      if(arr[idx] == t) {
        ctx.fillRect(
          col * (CELL_SIZE + 1) + 1,
          row * (CELL_SIZE + 1) + 1,
          CELL_SIZE,
          CELL_SIZE
        );
      }
    }
  }
};

const but = document.getElementById("run-toggle");

const play = () => {
  but.textContent = "⏸";
  renderLoop();
};

const pause = () => {
  but.textContent = "▶";
  cancelAnimationFrame(animationId);
  animationId = null;
};

const isPaused = () => {
  return animationId == null;
};

but.addEventListener('click', () => {
  if(isPaused()) {
    play();
  } else {
    pause();
  }
});

can.addEventListener('click', event => {

  console.log("Heeeeeyyyyooo");

  const boundingRect = can.getBoundingClientRect();

  const scaleX = can.width / boundingRect.width;
  const scaleY = can.height / boundingRect.height;

  const canvasLeft = (event.clientX - boundingRect.left) * scaleX;
  const canvasTop = (event.clientY - boundingRect.top) * scaleY;

  const row = Math.min(Math.floor(canvasTop / (CELL_SIZE + 1)), height - 1);
  const col = Math.min(Math.floor(canvasLeft / (CELL_SIZE + 1)), width - 1);

  console.log(row);
  console.log(col);

  if(event.shiftKey) {
    // universe.insert_glider(row, col);
    wasm.insert_glider(row, col);
  } else if(event.altKey) {
    // universe.insert_pulsar(row, col);
    wasm.insert_pulsar(row, col);
  } else {
    wasm.toggle(row, col);
    // universe.toggle(row, col);
  }

  drawGrid();
  drawCells();

});

const killAll = document.getElementById("killer");

killAll.addEventListener('click', () => {

  // universe.set_dimensions(width, height);
  wasm.kill_all();

});

const randomizer = document.getElementById("randomize");

randomizer.addEventListener('click', () => {

  console.log("rdmise");

  // universe.randomize_state();
  wasm.randomize_state();
  drawGrid();
  drawCells();

});

const ticker = document.getElementById("ticks-per-frame");

ticker.addEventListener("input", () => {
  console.log("Fired")
  if(ticker.valueAsNumber > 0) {
    tick_count = ticker.valueAsNumber;
  }
});

ticker.value = tick_count;

play();

// wasm.greet("Ameya Kore");
