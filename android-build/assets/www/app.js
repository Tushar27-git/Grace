/**
 * Logic Lab Mobile Engine (Android Edition)
 * Event-Driven 4-State Digital Simulation & Vector Canvas
 */

(function () {
  "use strict";

  // --- Constants & Color Tokens ---
  const COLOR_CANVAS = "#151217";
  const COLOR_PANEL = "#1E1A21";
  const COLOR_BORDER = "#6E4C9E";
  const COLOR_PINK = "#FF4FA3";
  const COLOR_PINK_GLOW = "rgba(255, 79, 163, 0.4)";
  const COLOR_LOW = "#4A4550";
  const COLOR_TEXT = "#EDEAF0";
  const COLOR_GRID = "#221E27";

  const SIGNAL_ZERO = 0;
  const SIGNAL_ONE = 1;
  const SIGNAL_X = 2; // unknown

  // --- Component Type Definitions ---
  const COMPONENT_DEFS = {
    // Basic Gates
    and: { name: "AND", inputs: 2, outputs: 1, cat: "gates", icon: "⊼", w: 56, h: 44 },
    or: { name: "OR", inputs: 2, outputs: 1, cat: "gates", icon: "≥1", w: 56, h: 44 },
    not: { name: "NOT", inputs: 1, outputs: 1, cat: "gates", icon: "1", w: 50, h: 36 },
    nand: { name: "NAND", inputs: 2, outputs: 1, cat: "gates", icon: "⊼", w: 58, h: 44 },
    nor: { name: "NOR", inputs: 2, outputs: 1, cat: "gates", icon: "⊽", w: 58, h: 44 },
    xor: { name: "XOR", inputs: 2, outputs: 1, cat: "gates", icon: "=1", w: 58, h: 44 },
    xnor: { name: "XNOR", inputs: 2, outputs: 1, cat: "gates", icon: "=", w: 60, h: 44 },
    buf: { name: "BUFFER", inputs: 1, outputs: 1, cat: "gates", icon: "▷", w: 50, h: 36 },

    // Input / Output
    switch: { name: "Switch", inputs: 0, outputs: 1, cat: "io", icon: "⏻", w: 48, h: 36 },
    button: { name: "Button", inputs: 0, outputs: 1, cat: "io", icon: "🔘", w: 48, h: 36 },
    clock: { name: "Clock", inputs: 0, outputs: 1, cat: "io", icon: "⏱", w: 48, h: 36 },
    led: { name: "LED", inputs: 1, outputs: 0, cat: "io", icon: "💡", w: 44, h: 36 },
    seven_seg: { name: "7-Segment", inputs: 7, outputs: 0, cat: "io", icon: "8", w: 52, h: 72 },
    hex_disp: { name: "Hex Display", inputs: 4, outputs: 0, cat: "io", icon: "🅵", w: 50, h: 54 },

    // Arithmetic
    half_adder: { name: "Half Adder", inputs: 2, outputs: 2, cat: "arithmetic", icon: "HA", w: 64, h: 48 },
    full_adder: { name: "Full Adder", inputs: 3, outputs: 2, cat: "arithmetic", icon: "FA", w: 66, h: 54 },
    half_sub: { name: "Half Sub", inputs: 2, outputs: 2, cat: "arithmetic", icon: "HS", w: 64, h: 48 },
    full_sub: { name: "Full Sub", inputs: 3, outputs: 2, cat: "arithmetic", icon: "FS", w: 66, h: 54 },
    adder4: { name: "Adder-4", inputs: 9, outputs: 5, cat: "arithmetic", icon: "Σ4", w: 84, h: 80 },

    // Sequential
    d_ff: { name: "D Flip-Flop", inputs: 2, outputs: 2, cat: "sequential", icon: "D-FF", w: 64, h: 54 },
    jk_ff: { name: "JK Flip-Flop", inputs: 3, outputs: 2, cat: "sequential", icon: "JK", w: 64, h: 58 },
    t_ff: { name: "T Flip-Flop", inputs: 2, outputs: 2, cat: "sequential", icon: "T-FF", w: 64, h: 54 },
    counter4: { name: "Counter-4", inputs: 2, outputs: 4, cat: "sequential", icon: "CNT", w: 74, h: 64 },
    register4: { name: "Register-4", inputs: 6, outputs: 4, cat: "sequential", icon: "REG", w: 76, h: 72 },

    // Data / Routing
    mux2: { name: "MUX 2:1", inputs: 3, outputs: 1, cat: "routing", icon: "MUX", w: 60, h: 54 },
    mux4: { name: "MUX 4:1", inputs: 6, outputs: 1, cat: "routing", icon: "MUX", w: 66, h: 72 },
    demux2: { name: "DEMUX 1:2", inputs: 2, outputs: 2, cat: "routing", icon: "DMX", w: 60, h: 54 },
    comparator4: { name: "Comparator", inputs: 8, outputs: 3, cat: "routing", icon: "CMP", w: 80, h: 80 },
  };

  // --- App State ---
  const state = {
    components: [],
    wires: [],
    nextId: 1,

    // Camera
    camera: { x: 0, y: 0, zoom: 1.0 },

    // Interaction
    currentTool: "select", // "select" | "wire" | "pan"
    selectedComp: null,
    selectedWire: null,
    selectedPaletteType: null,

    // Wire in progress
    wireDrag: null, // { sourceComp, sourcePort, curX, curY }

    // Multi-touch gestures
    touchStartDist: 0,
    touchStartZoom: 1.0,
    isPanning: false,
    dragStartPos: { x: 0, y: 0 },
    dragCompStartPos: { x: 0, y: 0 },

    // Simulation
    simRunning: true,
    clockState: false,
    clockFreq: 2, // Hz
    clockTimer: null,
    snapToGrid: true,

    // History for Undo
    undoStack: [],
    redoStack: [],

    // Waveform history
    waveHistory: {
      clock: [],
      inputs: [],
      outputs: []
    }
  };

  // --- Canvas Setup ---
  const canvas = document.getElementById("circuit-canvas");
  const ctx = canvas.getContext("2d");
  const viewport = document.getElementById("canvas-viewport");

  function resizeCanvas() {
    const rect = viewport.getBoundingClientRect();
    const dpr = window.devicePixelRatio || 1;
    canvas.width = rect.width * dpr;
    canvas.height = rect.height * dpr;
    ctx.scale(dpr, dpr);
    render();
  }
  window.addEventListener("resize", resizeCanvas);

  // Screen to Canvas Coordinates
  function screenToCanvas(sx, sy) {
    const rect = canvas.getBoundingClientRect();
    const px = sx - rect.left;
    const py = sy - rect.top;
    return {
      x: (px - canvas.clientWidth / 2) / state.camera.zoom - state.camera.x,
      y: (py - canvas.clientHeight / 2) / state.camera.zoom - state.camera.y
    };
  }

  // Canvas to Screen Coordinates
  function canvasToScreen(cx, cy) {
    return {
      x: (cx + state.camera.x) * state.camera.zoom + canvas.clientWidth / 2,
      y: (cy + state.camera.y) * state.camera.zoom + canvas.clientHeight / 2
    };
  }

  function snap(val, step = 20) {
    return state.snapToGrid ? Math.round(val / step) * step : val;
  }

  // --- Snapshot History for Undo/Redo ---
  function saveSnapshot() {
    const snapData = {
      components: JSON.parse(JSON.stringify(state.components)),
      wires: JSON.parse(JSON.stringify(state.wires)),
      nextId: state.nextId
    };
    state.undoStack.push(snapData);
    if (state.undoStack.length > 30) state.undoStack.shift();
    state.redoStack = [];
  }

  function undo() {
    if (state.undoStack.length === 0) return;
    const currentSnap = {
      components: JSON.parse(JSON.stringify(state.components)),
      wires: JSON.parse(JSON.stringify(state.wires)),
      nextId: state.nextId
    };
    state.redoStack.push(currentSnap);
    const prev = state.undoStack.pop();
    state.components = prev.components;
    state.wires = prev.wires;
    state.nextId = prev.nextId;
    state.selectedComp = null;
    updateSelectionBar();
    settleSimulation();
    render();
  }

  function redo() {
    if (state.redoStack.length === 0) return;
    const next = state.redoStack.pop();
    state.undoStack.push({
      components: JSON.parse(JSON.stringify(state.components)),
      wires: JSON.parse(JSON.stringify(state.wires)),
      nextId: state.nextId
    });
    state.components = next.components;
    state.wires = next.wires;
    state.nextId = next.nextId;
    state.selectedComp = null;
    updateSelectionBar();
    settleSimulation();
    render();
  }

  // --- Component Factory ---
  function createComponent(type, x, y) {
    const def = COMPONENT_DEFS[type];
    if (!def) return null;

    const comp = {
      id: state.nextId++,
      type: type,
      name: def.name,
      x: snap(x),
      y: snap(y),
      w: def.w,
      h: def.h,
      rotation: 0, // 0, 90, 180, 270
      inputs: new Array(def.inputs).fill(SIGNAL_ZERO),
      outputs: new Array(def.outputs).fill(SIGNAL_ZERO),
      stateFlag: false, // For toggle switches, button pressed state, or internal flip-flop state
      stateVal: 0 // For 4-bit registers/counters
    };

    return comp;
  }

  // --- Port Absolute Position Helpers ---
  function getPortPositions(comp) {
    const def = COMPONENT_DEFS[comp.type];
    const inPorts = [];
    const outPorts = [];

    const inCount = def.inputs;
    const outCount = def.outputs;

    for (let i = 0; i < inCount; i++) {
      const yOff = inCount === 1 ? 0 : (i / (inCount - 1) - 0.5) * (comp.h - 16);
      inPorts.push({
        index: i,
        isOutput: false,
        x: comp.x - comp.w / 2,
        y: comp.y + yOff
      });
    }

    for (let i = 0; i < outCount; i++) {
      const yOff = outCount === 1 ? 0 : (i / (outCount - 1) - 0.5) * (comp.h - 16);
      outPorts.push({
        index: i,
        isOutput: true,
        x: comp.x + comp.w / 2,
        y: comp.y + yOff
      });
    }

    // Apply rotation if needed
    if (comp.rotation !== 0) {
      const rad = (comp.rotation * Math.PI) / 180;
      const cos = Math.cos(rad);
      const sin = Math.sin(rad);

      const rotatePoint = (pt) => {
        const dx = pt.x - comp.x;
        const dy = pt.y - comp.y;
        pt.x = comp.x + dx * cos - dy * sin;
        pt.y = comp.y + dx * sin + dy * cos;
      };

      inPorts.forEach(rotatePoint);
      outPorts.forEach(rotatePoint);
    }

    return { inPorts, outPorts };
  }

  // --- Simulation Engine ---
  function evalComponent(comp) {
    const ins = comp.inputs;
    const outs = comp.outputs;

    switch (comp.type) {
      case "and":
        outs[0] = ins.every((s) => s === SIGNAL_ONE) ? SIGNAL_ONE : SIGNAL_ZERO;
        break;
      case "or":
        outs[0] = ins.some((s) => s === SIGNAL_ONE) ? SIGNAL_ONE : SIGNAL_ZERO;
        break;
      case "not":
        outs[0] = ins[0] === SIGNAL_ONE ? SIGNAL_ZERO : SIGNAL_ONE;
        break;
      case "nand":
        outs[0] = ins.every((s) => s === SIGNAL_ONE) ? SIGNAL_ZERO : SIGNAL_ONE;
        break;
      case "nor":
        outs[0] = ins.some((s) => s === SIGNAL_ONE) ? SIGNAL_ZERO : SIGNAL_ONE;
        break;
      case "xor": {
        let ones = ins.filter((s) => s === SIGNAL_ONE).length;
        outs[0] = ones % 2 === 1 ? SIGNAL_ONE : SIGNAL_ZERO;
        break;
      }
      case "xnor": {
        let ones = ins.filter((s) => s === SIGNAL_ONE).length;
        outs[0] = ones % 2 === 0 ? SIGNAL_ONE : SIGNAL_ZERO;
        break;
      }
      case "buf":
        outs[0] = ins[0];
        break;
      case "switch":
      case "button":
        outs[0] = comp.stateFlag ? SIGNAL_ONE : SIGNAL_ZERO;
        break;
      case "clock":
        outs[0] = state.clockState ? SIGNAL_ONE : SIGNAL_ZERO;
        break;
      case "led":
        // Display only
        break;
      case "half_adder": {
        const a = ins[0], b = ins[1];
        outs[0] = (a ^ b); // Sum
        outs[1] = (a & b); // Carry
        break;
      }
      case "full_adder": {
        const a = ins[0], b = ins[1], cin = ins[2];
        outs[0] = (a ^ b ^ cin); // Sum
        outs[1] = ((a & b) | (cin & (a ^ b))); // Cout
        break;
      }
      case "half_sub": {
        const a = ins[0], b = ins[1];
        outs[0] = (a ^ b); // Diff
        outs[1] = (!a & b); // Borrow
        break;
      }
      case "full_sub": {
        const a = ins[0], b = ins[1], bin = ins[2];
        outs[0] = (a ^ b ^ bin); // Diff
        outs[1] = ((!a & b) | (bin & !(a ^ b))); // Bout
        break;
      }
      case "adder4": {
        let valA = (ins[0] || 0) | ((ins[1] || 0) << 1) | ((ins[2] || 0) << 2) | ((ins[3] || 0) << 3);
        let valB = (ins[4] || 0) | ((ins[5] || 0) << 1) | ((ins[6] || 0) << 2) | ((ins[7] || 0) << 3);
        let cin = ins[8] || 0;
        let sum = valA + valB + cin;
        for (let i = 0; i < 4; i++) outs[i] = (sum >> i) & 1;
        outs[4] = (sum >> 4) & 1; // Cout
        break;
      }
      case "d_ff": {
        // Output Q and Q_bar
        outs[0] = comp.stateVal & 1;
        outs[1] = (comp.stateVal & 1) ? 0 : 1;
        break;
      }
      case "jk_ff":
      case "t_ff": {
        outs[0] = comp.stateVal & 1;
        outs[1] = (comp.stateVal & 1) ? 0 : 1;
        break;
      }
      case "counter4": {
        for (let i = 0; i < 4; i++) {
          outs[i] = (comp.stateVal >> i) & 1;
        }
        break;
      }
      case "register4": {
        for (let i = 0; i < 4; i++) {
          outs[i] = (comp.stateVal >> i) & 1;
        }
        break;
      }
      case "mux2": {
        const d0 = ins[0], d1 = ins[1], sel = ins[2];
        outs[0] = sel === 1 ? d1 : d0;
        break;
      }
      case "mux4": {
        const d = [ins[0], ins[1], ins[2], ins[3]];
        const sel = (ins[4] || 0) | ((ins[5] || 0) << 1);
        outs[0] = d[sel] || 0;
        break;
      }
      case "demux2": {
        const inp = ins[0], sel = ins[1];
        outs[0] = sel === 0 ? inp : 0;
        outs[1] = sel === 1 ? inp : 0;
        break;
      }
      case "comparator4": {
        let valA = (ins[0] || 0) | ((ins[1] || 0) << 1) | ((ins[2] || 0) << 2) | ((ins[3] || 0) << 3);
        let valB = (ins[4] || 0) | ((ins[5] || 0) << 1) | ((ins[6] || 0) << 2) | ((ins[7] || 0) << 3);
        outs[0] = valA > valB ? 1 : 0;
        outs[1] = valA === valB ? 1 : 0;
        outs[2] = valA < valB ? 1 : 0;
        break;
      }
    }
  }

  function settleSimulation() {
    const compMap = new Map(state.components.map((c) => [c.id, c]));

    // Synchronize initial component outputs
    for (let c of state.components) {
      evalComponent(c);
    }

    // Event-driven propagation loop
    let changed = true;
    let iterations = 0;
    const maxIterations = state.components.length * 15 + 50;

    while (changed && iterations < maxIterations) {
      changed = false;
      iterations++;

      // Propagate signals across wires
      for (let wire of state.wires) {
        const src = compMap.get(wire.fromCompId);
        const dst = compMap.get(wire.toCompId);

        if (src && dst) {
          const sig = src.outputs[wire.fromPort] || SIGNAL_ZERO;
          wire.currentSignal = sig;

          if (dst.inputs[wire.toPort] !== sig) {
            dst.inputs[wire.toPort] = sig;
            changed = true;
          }
        }
      }

      // Re-evaluate affected components
      for (let c of state.components) {
        const oldOuts = [...c.outputs];
        evalComponent(c);
        for (let i = 0; i < oldOuts.length; i++) {
          if (oldOuts[i] !== c.outputs[i]) {
            changed = true;
          }
        }
      }
    }

    // Update Waveform samples if any active
    sampleWaveforms();
  }

  function tickSimulation() {
    // Clock edge pulse
    state.clockState = !state.clockState;
    document.getElementById("clock-led").classList.toggle("high", state.clockState);

    // On rising edge, advance sequential elements
    if (state.clockState) {
      for (let comp of state.components) {
        if (comp.type === "d_ff") {
          // Input D is ins[0], Clock is ins[1]
          comp.stateVal = comp.inputs[0];
        } else if (comp.type === "jk_ff") {
          const j = comp.inputs[0], k = comp.inputs[1];
          const q = comp.stateVal & 1;
          if (j === 0 && k === 1) comp.stateVal = 0;
          else if (j === 1 && k === 0) comp.stateVal = 1;
          else if (j === 1 && k === 1) comp.stateVal = q ? 0 : 1;
        } else if (comp.type === "t_ff") {
          const t = comp.inputs[0];
          if (t === 1) comp.stateVal = (comp.stateVal & 1) ? 0 : 1;
        } else if (comp.type === "counter4") {
          const rst = comp.inputs[1];
          if (rst === 1) comp.stateVal = 0;
          else comp.stateVal = (comp.stateVal + 1) & 0x0f;
        } else if (comp.type === "register4") {
          const load = comp.inputs[4];
          const clr = comp.inputs[5];
          if (clr === 1) comp.stateVal = 0;
          else if (load === 1) {
            comp.stateVal = (comp.inputs[0] || 0) | ((comp.inputs[1] || 0) << 1) | ((comp.inputs[2] || 0) << 2) | ((comp.inputs[3] || 0) << 3);
          }
        }
      }
    }

    settleSimulation();
    render();
  }

  function startSimTimer() {
    if (state.clockTimer) clearInterval(state.clockTimer);
    const intervalMs = 1000 / (state.clockFreq * 2);
    state.clockTimer = setInterval(() => {
      if (state.simRunning) {
        tickSimulation();
      }
    }, intervalMs);
  }

  // --- Rendering Functions ---
  function render() {
    ctx.clearRect(0, 0, canvas.width, canvas.height);

    ctx.save();
    // Center camera
    ctx.translate(canvas.clientWidth / 2, canvas.clientHeight / 2);
    ctx.scale(state.camera.zoom, state.camera.zoom);
    ctx.translate(state.camera.x, state.camera.y);

    drawGrid();
    drawWires();
    drawComponents();

    // Wire in progress
    if (state.wireDrag) {
      drawWireInProgress();
    }

    ctx.restore();
  }

  function drawGrid() {
    const zoom = state.camera.zoom;
    const step = 20;
    const viewW = canvas.clientWidth / zoom;
    const viewH = canvas.clientHeight / zoom;

    const left = -state.camera.x - viewW / 2;
    const top = -state.camera.y - viewH / 2;
    const right = left + viewW;
    const bottom = top + viewH;

    const startX = Math.floor(left / step) * step;
    const startY = Math.floor(top / step) * step;

    ctx.fillStyle = COLOR_GRID;
    for (let x = startX; x <= right; x += step) {
      for (let y = startY; y <= bottom; y += step) {
        ctx.fillRect(x - 1, y - 1, 2, 2);
      }
    }
  }

  function drawWires() {
    const compMap = new Map(state.components.map((c) => [c.id, c]));

    for (let wire of state.wires) {
      const src = compMap.get(wire.fromCompId);
      const dst = compMap.get(wire.toCompId);
      if (!src || !dst) continue;

      const srcPorts = getPortPositions(src).outPorts;
      const dstPorts = getPortPositions(dst).inPorts;

      const sp = srcPorts[wire.fromPort];
      const dp = dstPorts[wire.toPort];
      if (!sp || !dp) continue;

      const isHigh = wire.currentSignal === SIGNAL_ONE;
      const isSelected = state.selectedWire === wire;

      ctx.beginPath();
      ctx.moveTo(sp.x, sp.y);

      // Orthogonal step routing
      const midX = (sp.x + dp.x) / 2;
      ctx.lineTo(midX, sp.y);
      ctx.lineTo(midX, dp.y);
      ctx.lineTo(dp.x, dp.y);

      ctx.lineWidth = isSelected ? 3.5 : isHigh ? 2.5 : 1.8;
      ctx.strokeStyle = isSelected ? "#fff" : isHigh ? COLOR_PINK : COLOR_LOW;

      if (isHigh) {
        ctx.shadowColor = COLOR_PINK;
        ctx.shadowBlur = 8;
      }
      ctx.stroke();
      ctx.shadowBlur = 0;

      // Junction dot at output pin
      ctx.beginPath();
      ctx.arc(sp.x, sp.y, 3, 0, Math.PI * 2);
      ctx.fillStyle = isHigh ? COLOR_PINK : COLOR_LOW;
      ctx.fill();
    }
  }

  function drawWireInProgress() {
    const wd = state.wireDrag;
    ctx.beginPath();
    ctx.moveTo(wd.startX, wd.startY);
    const midX = (wd.startX + wd.curX) / 2;
    ctx.lineTo(midX, wd.startY);
    ctx.lineTo(midX, wd.curY);
    ctx.lineTo(wd.curX, wd.curY);

    ctx.lineWidth = 2.5;
    ctx.strokeStyle = COLOR_PINK;
    ctx.setLineDash([6, 4]);
    ctx.stroke();
    ctx.setLineDash([]);
  }

  function drawComponents() {
    for (let comp of state.components) {
      ctx.save();
      ctx.translate(comp.x, comp.y);
      if (comp.rotation !== 0) {
        ctx.rotate((comp.rotation * Math.PI) / 180);
      }

      const isSelected = state.selectedComp === comp;

      // Draw Gate Body according to type
      drawGateGlyph(comp, isSelected);

      // Draw Pins & Stubs
      drawGatePins(comp);

      ctx.restore();
    }
  }

  function drawGateGlyph(comp, isSelected) {
    const w = comp.w;
    const h = comp.h;
    const hw = w / 2;
    const hh = h / 2;

    ctx.fillStyle = COLOR_PANEL;
    ctx.strokeStyle = isSelected ? COLOR_PINK : COLOR_BORDER;
    ctx.lineWidth = isSelected ? 2.5 : 1.5;

    if (isSelected) {
      ctx.shadowColor = COLOR_PINK_GLOW;
      ctx.shadowBlur = 12;
    }

    switch (comp.type) {
      case "and":
      case "nand": {
        // Flat-back D shape
        ctx.beginPath();
        ctx.moveTo(-hw, -hh);
        ctx.lineTo(0, -hh);
        ctx.arc(0, 0, hh, -Math.PI / 2, Math.PI / 2);
        ctx.lineTo(-hw, hh);
        ctx.closePath();
        ctx.fill();
        ctx.stroke();

        if (comp.type === "nand") {
          drawBubble(hw - 4, 0);
        }
        break;
      }
      case "or":
      case "nor": {
        // Curved-back shield shape
        ctx.beginPath();
        ctx.moveTo(-hw, -hh);
        ctx.quadraticCurveTo(-hw * 0.4, 0, -hw, hh);
        ctx.quadraticCurveTo(0, hh, hw - 6, 0);
        ctx.quadraticCurveTo(0, -hh, -hw, -hh);
        ctx.closePath();
        ctx.fill();
        ctx.stroke();

        if (comp.type === "nor") {
          drawBubble(hw - 4, 0);
        }
        break;
      }
      case "xor":
      case "xnor": {
        // Curved leading line + OR body
        ctx.beginPath();
        ctx.moveTo(-hw + 6, -hh);
        ctx.quadraticCurveTo(-hw * 0.4 + 6, 0, -hw + 6, hh);
        ctx.quadraticCurveTo(0, hh, hw - 6, 0);
        ctx.quadraticCurveTo(0, -hh, -hw + 6, -hh);
        ctx.closePath();
        ctx.fill();
        ctx.stroke();

        // Extra back line
        ctx.beginPath();
        ctx.moveTo(-hw, -hh);
        ctx.quadraticCurveTo(-hw * 0.4, 0, -hw, hh);
        ctx.stroke();

        if (comp.type === "xnor") {
          drawBubble(hw - 4, 0);
        }
        break;
      }
      case "not":
      case "buf": {
        // Triangle shape
        ctx.beginPath();
        ctx.moveTo(-hw, -hh);
        ctx.lineTo(hw - (comp.type === "not" ? 10 : 0), 0);
        ctx.lineTo(-hw, hh);
        ctx.closePath();
        ctx.fill();
        ctx.stroke();

        if (comp.type === "not") {
          drawBubble(hw - 6, 0);
        }
        break;
      }
      case "switch": {
        // Toggle Switch housing
        drawRoundedRect(-hw, -hh, w, h, 6);
        ctx.fill();
        ctx.stroke();

        // Switch toggle lever
        const isOn = comp.stateFlag;
        ctx.beginPath();
        ctx.arc(isOn ? hw - 14 : -hw + 14, 0, 7, 0, Math.PI * 2);
        ctx.fillStyle = isOn ? COLOR_PINK : COLOR_LOW;
        if (isOn) {
          ctx.shadowColor = COLOR_PINK;
          ctx.shadowBlur = 8;
        }
        ctx.fill();
        ctx.shadowBlur = 0;
        break;
      }
      case "button": {
        drawRoundedRect(-hw, -hh, w, h, 6);
        ctx.fill();
        ctx.stroke();

        // Button circle
        ctx.beginPath();
        ctx.arc(0, 0, 8, 0, Math.PI * 2);
        ctx.fillStyle = comp.stateFlag ? COLOR_PINK : COLOR_BORDER;
        ctx.fill();
        break;
      }
      case "clock": {
        drawRoundedRect(-hw, -hh, w, h, 6);
        ctx.fill();
        ctx.stroke();

        // Square wave icon
        ctx.strokeStyle = state.clockState ? COLOR_PINK : COLOR_TEXT;
        ctx.lineWidth = 1.5;
        ctx.beginPath();
        ctx.moveTo(-10, 4);
        ctx.lineTo(-10, -4);
        ctx.lineTo(0, -4);
        ctx.lineTo(0, 4);
        ctx.lineTo(10, 4);
        ctx.lineTo(10, -4);
        ctx.stroke();
        break;
      }
      case "led": {
        drawRoundedRect(-hw, -hh, w, h, 6);
        ctx.fill();
        ctx.stroke();

        const isLit = comp.inputs[0] === SIGNAL_ONE;
        ctx.beginPath();
        ctx.arc(0, 0, 10, 0, Math.PI * 2);
        ctx.fillStyle = isLit ? COLOR_PINK : "#221d28";
        if (isLit) {
          ctx.shadowColor = COLOR_PINK;
          ctx.shadowBlur = 14;
        }
        ctx.fill();
        ctx.shadowBlur = 0;
        ctx.strokeStyle = isLit ? "#fff" : COLOR_BORDER;
        ctx.stroke();
        break;
      }
      case "hex_disp": {
        drawRoundedRect(-hw, -hh, w, h, 6);
        ctx.fill();
        ctx.stroke();

        // Compute 4-bit hex value
        const val = (comp.inputs[0] || 0) | ((comp.inputs[1] || 0) << 1) | ((comp.inputs[2] || 0) << 2) | ((comp.inputs[3] || 0) << 3);
        const hexChar = val.toString(16).toUpperCase();

        ctx.font = "bold 24px monospace";
        ctx.fillStyle = COLOR_PINK;
        ctx.textAlign = "center";
        ctx.textBaseline = "middle";
        ctx.fillText(hexChar, 0, 0);
        break;
      }
      case "seven_seg": {
        drawRoundedRect(-hw, -hh, w, h, 6);
        ctx.fill();
        ctx.stroke();

        // 7-segment display logic
        ctx.font = "bold 10px monospace";
        ctx.fillStyle = COLOR_TEXT;
        ctx.textAlign = "center";
        ctx.fillText("7-SEG", 0, -hh + 12);
        break;
      }
      default: {
        // Generic chip block
        drawRoundedRect(-hw, -hh, w, h, 6);
        ctx.fill();
        ctx.stroke();

        // Label
        ctx.font = "bold 11px sans-serif";
        ctx.fillStyle = COLOR_TEXT;
        ctx.textAlign = "center";
        ctx.textBaseline = "middle";
        ctx.fillText(comp.name, 0, 0);
        break;
      }
    }

    ctx.shadowBlur = 0;
  }

  function drawBubble(x, y) {
    ctx.beginPath();
    ctx.arc(x, y, 3.5, 0, Math.PI * 2);
    ctx.fillStyle = COLOR_PANEL;
    ctx.fill();
    ctx.stroke();
  }

  function drawRoundedRect(x, y, w, h, r) {
    ctx.beginPath();
    ctx.moveTo(x + r, y);
    ctx.lineTo(x + w - r, y);
    ctx.quadraticCurveTo(x + w, y, x + w, y + r);
    ctx.lineTo(x + w, y + h - r);
    ctx.quadraticCurveTo(x + w, y + h, x + w - r, y + h);
    ctx.lineTo(x + r, y + h);
    ctx.quadraticCurveTo(x, y + h, x, y + h - r);
    ctx.lineTo(x, y + r);
    ctx.quadraticCurveTo(x, y, x + r, y);
    ctx.closePath();
  }

  function drawGatePins(comp) {
    const def = COMPONENT_DEFS[comp.type];
    const hw = comp.w / 2;
    const hh = comp.h / 2;

    const inCount = def.inputs;
    const outCount = def.outputs;

    // Draw input pins
    for (let i = 0; i < inCount; i++) {
      const yOff = inCount === 1 ? 0 : (i / (inCount - 1) - 0.5) * (comp.h - 16);
      const isHigh = comp.inputs[i] === SIGNAL_ONE;

      ctx.beginPath();
      ctx.moveTo(-hw - 8, yOff);
      ctx.lineTo(-hw, yOff);
      ctx.strokeStyle = isHigh ? COLOR_PINK : COLOR_LOW;
      ctx.lineWidth = 2;
      ctx.stroke();

      ctx.beginPath();
      ctx.arc(-hw - 8, yOff, 3, 0, Math.PI * 2);
      ctx.fillStyle = isHigh ? COLOR_PINK : COLOR_LOW;
      ctx.fill();
    }

    // Draw output pins
    for (let i = 0; i < outCount; i++) {
      const yOff = outCount === 1 ? 0 : (i / (outCount - 1) - 0.5) * (comp.h - 16);
      const isHigh = comp.outputs[i] === SIGNAL_ONE;

      ctx.beginPath();
      ctx.moveTo(hw, yOff);
      ctx.lineTo(hw + 8, yOff);
      ctx.strokeStyle = isHigh ? COLOR_PINK : COLOR_LOW;
      ctx.lineWidth = 2;
      ctx.stroke();

      ctx.beginPath();
      ctx.arc(hw + 8, yOff, 3, 0, Math.PI * 2);
      ctx.fillStyle = isHigh ? COLOR_PINK : COLOR_LOW;
      ctx.fill();
    }
  }

  // --- Interaction & Event Handlers ---
  function findComponentAt(cx, cy) {
    for (let i = state.components.length - 1; i >= 0; i--) {
      const c = state.components[i];
      if (Math.abs(cx - c.x) <= c.w / 2 && Math.abs(cy - c.y) <= c.h / 2) {
        return c;
      }
    }
    return null;
  }

  function findPortAt(cx, cy, threshold = 16) {
    for (let c of state.components) {
      const { inPorts, outPorts } = getPortPositions(c);
      for (let p of outPorts) {
        if (Math.hypot(cx - p.x, cy - p.y) <= threshold) {
          return { comp: c, port: p };
        }
      }
      for (let p of inPorts) {
        if (Math.hypot(cx - p.x, cy - p.y) <= threshold) {
          return { comp: c, port: p };
        }
      }
    }
    return null;
  }

  function updateSelectionBar() {
    const bar = document.getElementById("selection-bar");
    if (state.selectedComp) {
      bar.classList.add("show");
    } else {
      bar.classList.remove("show");
    }
  }

  // Pointer / Touch Events
  let isPointerDown = false;
  let pointerStartPos = { x: 0, y: 0 };
  let hasMoved = false;

  canvas.addEventListener("pointerdown", (e) => {
    isPointerDown = true;
    hasMoved = false;
    const pos = screenToCanvas(e.clientX, e.clientY);
    pointerStartPos = { x: e.clientX, y: e.clientY };

    // Check if clicked near a port to start wiring
    const portHit = findPortAt(pos.x, pos.y);
    if (portHit && portHit.port.isOutput) {
      state.wireDrag = {
        fromCompId: portHit.comp.id,
        fromPort: portHit.port.index,
        startX: portHit.port.x,
        startY: portHit.port.y,
        curX: pos.x,
        curY: pos.y
      };
      render();
      return;
    }

    // Check if hit a component
    const compHit = findComponentAt(pos.x, pos.y);
    if (compHit) {
      state.selectedComp = compHit;
      state.selectedWire = null;
      updateSelectionBar();

      // Setup drag movement
      state.dragStartPos = { x: pos.x, y: pos.y };
      state.dragCompStartPos = { x: compHit.x, y: compHit.y };

      // Immediate toggle for switch or push button
      if (compHit.type === "switch") {
        compHit.stateFlag = !compHit.stateFlag;
        settleSimulation();
      } else if (compHit.type === "button") {
        compHit.stateFlag = true;
        settleSimulation();
      }

      render();
      return;
    }

    // Hit empty canvas
    if (state.selectedPaletteType) {
      // Place component
      saveSnapshot();
      const newComp = createComponent(state.selectedPaletteType, pos.x, pos.y);
      if (newComp) {
        state.components.push(newComp);
        state.selectedComp = newComp;
        updateSelectionBar();
        settleSimulation();
        render();
      }
      state.selectedPaletteType = null;
      document.querySelectorAll(".gate-card").forEach((c) => c.classList.remove("selected"));
      return;
    }

    // Otherwise initiate pan or clear selection
    state.selectedComp = null;
    state.selectedWire = null;
    updateSelectionBar();
    state.isPanning = true;
    state.dragStartPos = { x: e.clientX, y: e.clientY };
    render();
  });

  window.addEventListener("pointermove", (e) => {
    if (!isPointerDown) return;
    const dx = e.clientX - pointerStartPos.x;
    const dy = e.clientY - pointerStartPos.y;
    if (Math.hypot(dx, dy) > 5) hasMoved = true;

    const pos = screenToCanvas(e.clientX, e.clientY);

    // Update wire in progress
    if (state.wireDrag) {
      state.wireDrag.curX = pos.x;
      state.wireDrag.curY = pos.y;
      render();
      return;
    }

    // Drag moving selected component
    if (state.selectedComp && !state.isPanning) {
      const c = state.selectedComp;
      const cdx = pos.x - state.dragStartPos.x;
      const cdy = pos.y - state.dragStartPos.y;
      c.x = snap(state.dragCompStartPos.x + cdx);
      c.y = snap(state.dragCompStartPos.y + cdy);
      render();
      return;
    }

    // Panning canvas
    if (state.isPanning) {
      const pdx = (e.clientX - state.dragStartPos.x) / state.camera.zoom;
      const pdy = (e.clientY - state.dragStartPos.y) / state.camera.zoom;
      state.camera.x += pdx;
      state.camera.y += pdy;
      state.dragStartPos = { x: e.clientX, y: e.clientY };
      render();
    }
  });

  window.addEventListener("pointerup", (e) => {
    if (!isPointerDown) return;
    isPointerDown = false;
    state.isPanning = false;

    // Release momentary button
    if (state.selectedComp && state.selectedComp.type === "button") {
      state.selectedComp.stateFlag = false;
      settleSimulation();
      render();
    }

    // Finish wiring if active
    if (state.wireDrag) {
      const pos = screenToCanvas(e.clientX, e.clientY);
      const portHit = findPortAt(pos.x, pos.y);

      if (portHit && !portHit.port.isOutput && portHit.comp.id !== state.wireDrag.fromCompId) {
        saveSnapshot();
        // Remove any wire already connected to this input port
        state.wires = state.wires.filter(
          (w) => !(w.toCompId === portHit.comp.id && w.toPort === portHit.port.index)
        );

        // Add new connection
        state.wires.push({
          fromCompId: state.wireDrag.fromCompId,
          fromPort: state.wireDrag.fromPort,
          toCompId: portHit.comp.id,
          toPort: portHit.port.index,
          currentSignal: SIGNAL_ZERO
        });

        settleSimulation();
      }

      state.wireDrag = null;
      render();
    }
  });

  // Touch Pinch-to-Zoom
  viewport.addEventListener("touchstart", (e) => {
    if (e.touches.length === 2) {
      state.touchStartDist = Math.hypot(
        e.touches[0].clientX - e.touches[1].clientX,
        e.touches[0].clientY - e.touches[1].clientY
      );
      state.touchStartZoom = state.camera.zoom;
    }
  });

  viewport.addEventListener("touchmove", (e) => {
    if (e.touches.length === 2) {
      const dist = Math.hypot(
        e.touches[0].clientX - e.touches[1].clientX,
        e.touches[0].clientY - e.touches[1].clientY
      );
      if (state.touchStartDist > 0) {
        const factor = dist / state.touchStartDist;
        state.camera.zoom = Math.min(Math.max(state.touchStartZoom * factor, 0.3), 3.0);
        render();
      }
    }
  });

  // Mouse Wheel Zoom for tablets with mouse
  canvas.addEventListener("wheel", (e) => {
    e.preventDefault();
    const zoomFactor = e.deltaY < 0 ? 1.15 : 0.85;
    state.camera.zoom = Math.min(Math.max(state.camera.zoom * zoomFactor, 0.3), 3.0);
    render();
  });

  // --- Toolbar & Action Bindings ---
  document.getElementById("btn-sim-play").addEventListener("click", () => {
    state.simRunning = !state.simRunning;
    const btn = document.getElementById("btn-sim-play");
    btn.textContent = state.simRunning ? "▶ Run" : "⏸ Pause";
    btn.classList.toggle("active", state.simRunning);
  });

  document.getElementById("btn-sim-step").addEventListener("click", () => {
    tickSimulation();
  });

  document.getElementById("btn-sim-freq").addEventListener("click", () => {
    const freqs = [1, 2, 5, 10];
    const idx = (freqs.indexOf(state.clockFreq) + 1) % freqs.length;
    state.clockFreq = freqs[idx];
    document.getElementById("btn-sim-freq").textContent = `${state.clockFreq} Hz`;
    startSimTimer();
  });

  document.getElementById("btn-zoom-in").addEventListener("click", () => {
    state.camera.zoom = Math.min(state.camera.zoom * 1.25, 3.0);
    render();
  });

  document.getElementById("btn-zoom-out").addEventListener("click", () => {
    state.camera.zoom = Math.max(state.camera.zoom * 0.8, 0.3);
    render();
  });

  document.getElementById("btn-zoom-reset").addEventListener("click", () => {
    state.camera.x = 0;
    state.camera.y = 0;
    state.camera.zoom = 1.0;
    render();
  });

  document.getElementById("btn-grid-toggle").addEventListener("click", () => {
    state.snapToGrid = !state.snapToGrid;
    document.getElementById("btn-grid-toggle").classList.toggle("active", state.snapToGrid);
  });

  document.getElementById("btn-clear").addEventListener("click", () => {
    if (confirm("Clear entire circuit?")) {
      saveSnapshot();
      state.components = [];
      state.wires = [];
      state.selectedComp = null;
      updateSelectionBar();
      render();
    }
  });

  // Selection Bar Actions
  document.getElementById("sel-rotate").addEventListener("click", () => {
    if (state.selectedComp) {
      saveSnapshot();
      state.selectedComp.rotation = (state.selectedComp.rotation + 90) % 360;
      render();
    }
  });

  document.getElementById("sel-duplicate").addEventListener("click", () => {
    if (state.selectedComp) {
      saveSnapshot();
      const orig = state.selectedComp;
      const dup = createComponent(orig.type, orig.x + 40, orig.y + 40);
      dup.rotation = orig.rotation;
      state.components.push(dup);
      state.selectedComp = dup;
      updateSelectionBar();
      settleSimulation();
      render();
    }
  });

  document.getElementById("sel-delete").addEventListener("click", () => {
    if (state.selectedComp) {
      saveSnapshot();
      const id = state.selectedComp.id;
      state.components = state.components.filter((c) => c.id !== id);
      state.wires = state.wires.filter((w) => w.fromCompId !== id && w.toCompId !== id);
      state.selectedComp = null;
      updateSelectionBar();
      settleSimulation();
      render();
    }
  });

  document.getElementById("sel-close").addEventListener("click", () => {
    state.selectedComp = null;
    updateSelectionBar();
    render();
  });

  // Palette Categories
  function renderPalette(category) {
    const container = document.getElementById("palette-items");
    container.innerHTML = "";

    for (let key in COMPONENT_DEFS) {
      const def = COMPONENT_DEFS[key];
      if (def.cat !== category) continue;

      const card = document.createElement("div");
      card.className = "gate-card";
      if (state.selectedPaletteType === key) card.classList.add("selected");

      card.innerHTML = `
        <div class="gate-card-icon">${def.icon}</div>
        <div class="gate-card-label">${def.name}</div>
      `;

      card.addEventListener("click", () => {
        document.querySelectorAll(".gate-card").forEach((c) => c.classList.remove("selected"));
        if (state.selectedPaletteType === key) {
          state.selectedPaletteType = null;
        } else {
          state.selectedPaletteType = key;
          card.classList.add("selected");
        }
      });

      container.appendChild(card);
    }
  }

  document.querySelectorAll(".palette-tab").forEach((tab) => {
    tab.addEventListener("click", () => {
      document.querySelectorAll(".palette-tab").forEach((t) => t.classList.remove("active"));
      tab.classList.add("active");
      renderPalette(tab.dataset.cat);
    });
  });

  // --- Modals Management ---
  function openModal(id) {
    document.getElementById(id).classList.add("open");
  }

  function closeModal(id) {
    document.getElementById(id).classList.remove("open");
  }

  document.querySelectorAll(".modal-close-btn, .modal-close-action").forEach((btn) => {
    btn.addEventListener("click", (e) => {
      e.target.closest(".modal-overlay").classList.remove("open");
    });
  });

  // --- Truth Table Generator ---
  document.getElementById("btn-truth-table").addEventListener("click", () => {
    generateTruthTable();
    openModal("modal-truth-table");
  });

  function generateTruthTable() {
    const container = document.getElementById("truth-table-container");
    container.innerHTML = "";

    // Find input switches and output LEDs
    const switches = state.components.filter((c) => c.type === "switch" || c.type === "button");
    const leds = state.components.filter((c) => c.type === "led");

    if (switches.length === 0 || leds.length === 0) {
      container.innerHTML = `<p style="color:#ff8c8c; padding:10px;">Please wire at least one Toggle Switch and one LED to generate a truth table.</p>`;
      return;
    }

    if (switches.length > 8) {
      container.innerHTML = `<p style="color:#ff8c8c; padding:10px;">Maximum 8 inputs supported for exhaustive truth table.</p>`;
      return;
    }

    const rowsCount = 1 << switches.length;
    let html = `<table class="truth-table"><thead><tr>`;
    switches.forEach((s, idx) => (html += `<th>IN ${idx + 1}</th>`));
    leds.forEach((l, idx) => (html += `<th>OUT ${idx + 1}</th>`));
    html += `</tr></thead><tbody>`;

    const savedSwitchStates = switches.map((s) => s.stateFlag);

    for (let r = 0; r < rowsCount; r++) {
      // Set switches for row r
      for (let i = 0; i < switches.length; i++) {
        switches[i].stateFlag = ((r >> (switches.length - 1 - i)) & 1) === 1;
      }
      settleSimulation();

      html += `<tr class="tt-row" data-row="${r}">`;
      for (let i = 0; i < switches.length; i++) {
        const val = ((r >> (switches.length - 1 - i)) & 1);
        html += `<td>${val}</td>`;
      }
      for (let l of leds) {
        const outVal = l.inputs[0] === SIGNAL_ONE ? 1 : 0;
        html += `<td style="color:${outVal ? COLOR_PINK : 'inherit'}; font-weight:${outVal ? 'bold' : 'normal'}">${outVal}</td>`;
      }
      html += `</tr>`;
    }
    html += `</tbody></table>`;
    container.innerHTML = html;

    // Restore switch states
    switches.forEach((s, idx) => (s.stateFlag = savedSwitchStates[idx]));
    settleSimulation();

    // Row click drives circuit live
    container.querySelectorAll(".tt-row").forEach((tr) => {
      tr.addEventListener("click", () => {
        const r = parseInt(tr.dataset.row, 10);
        for (let i = 0; i < switches.length; i++) {
          switches[i].stateFlag = ((r >> (switches.length - 1 - i)) & 1) === 1;
        }
        settleSimulation();
        render();
      });
    });
  }

  // --- Universal Gates Lab ---
  document.getElementById("btn-universal-lab").addEventListener("click", () => {
    openModal("modal-universal-lab");
  });

  document.querySelectorAll(".btn-load-challenge").forEach((btn) => {
    btn.addEventListener("click", () => {
      const ch = btn.dataset.challenge;
      loadChallengeScaffold(ch);
      closeModal("modal-universal-lab");
    });
  });

  function loadChallengeScaffold(challenge) {
    saveSnapshot();
    state.components = [];
    state.wires = [];

    if (challenge === "not_nand") {
      const sw = createComponent("switch", -100, 0);
      const nand = createComponent("nand", 0, 0);
      const led = createComponent("led", 100, 0);
      state.components.push(sw, nand, led);

      // Connect both inputs of NAND to switch
      state.wires.push({ fromCompId: sw.id, fromPort: 0, toCompId: nand.id, toPort: 0, currentSignal: 0 });
      state.wires.push({ fromCompId: sw.id, fromPort: 0, toCompId: nand.id, toPort: 1, currentSignal: 0 });
      state.wires.push({ fromCompId: nand.id, fromPort: 0, toCompId: led.id, toPort: 0, currentSignal: 0 });
    } else if (challenge === "and_nand") {
      const sw1 = createComponent("switch", -140, -30);
      const sw2 = createComponent("switch", -140, 30);
      const nand1 = createComponent("nand", -30, 0);
      const nand2 = createComponent("nand", 70, 0);
      const led = createComponent("led", 160, 0);
      state.components.push(sw1, sw2, nand1, nand2, led);

      state.wires.push({ fromCompId: sw1.id, fromPort: 0, toCompId: nand1.id, toPort: 0, currentSignal: 0 });
      state.wires.push({ fromCompId: sw2.id, fromPort: 0, toCompId: nand1.id, toPort: 1, currentSignal: 0 });
      state.wires.push({ fromCompId: nand1.id, fromPort: 0, toCompId: nand2.id, toPort: 0, currentSignal: 0 });
      state.wires.push({ fromCompId: nand1.id, fromPort: 0, toCompId: nand2.id, toPort: 1, currentSignal: 0 });
      state.wires.push({ fromCompId: nand2.id, fromPort: 0, toCompId: led.id, toPort: 0, currentSignal: 0 });
    }

    state.camera.x = 0;
    state.camera.y = 0;
    settleSimulation();
    render();
  }

  // --- Waveform Viewer ---
  document.getElementById("btn-waveform").addEventListener("click", () => {
    openModal("modal-waveform");
    drawWaveforms();
  });

  function sampleWaveforms() {
    state.waveHistory.clock.push(state.clockState ? 1 : 0);
    const sw = state.components.find((c) => c.type === "switch");
    const led = state.components.find((c) => c.type === "led");

    state.waveHistory.inputs.push(sw ? (sw.stateFlag ? 1 : 0) : 0);
    state.waveHistory.outputs.push(led ? (led.inputs[0] === SIGNAL_ONE ? 1 : 0) : 0);

    if (state.waveHistory.clock.length > 50) {
      state.waveHistory.clock.shift();
      state.waveHistory.inputs.shift();
      state.waveHistory.outputs.shift();
    }
  }

  function drawWaveforms() {
    const wCanvas = document.getElementById("waveform-canvas");
    if (!wCanvas) return;
    const wCtx = wCanvas.getContext("2d");
    wCtx.clearRect(0, 0, wCanvas.width, wCanvas.height);

    const channels = [
      { name: "CLK", data: state.waveHistory.clock, color: COLOR_BORDER },
      { name: "IN", data: state.waveHistory.inputs, color: "#8E8798" },
      { name: "OUT", data: state.waveHistory.outputs, color: COLOR_PINK }
    ];

    const h = 50;
    channels.forEach((ch, idx) => {
      const topY = 20 + idx * 56;
      wCtx.fillStyle = COLOR_TEXT;
      wCtx.font = "bold 11px monospace";
      wCtx.fillText(ch.name, 10, topY + 20);

      wCtx.beginPath();
      wCtx.strokeStyle = ch.color;
      wCtx.lineWidth = 2;

      const data = ch.data;
      const stepX = (wCanvas.width - 70) / Math.max(data.length - 1, 1);

      for (let i = 0; i < data.length; i++) {
        const x = 60 + i * stepX;
        const y = topY + (data[i] === 1 ? 4 : 32);
        if (i === 0) wCtx.moveTo(x, y);
        else {
          const prevY = topY + (data[i - 1] === 1 ? 4 : 32);
          wCtx.lineTo(x, prevY);
          wCtx.lineTo(x, y);
        }
      }
      wCtx.stroke();
    });
  }

  // --- HDL Export ---
  document.getElementById("btn-hdl-export").addEventListener("click", () => {
    exportHDL();
    openModal("modal-hdl");
  });

  function exportHDL(isVhdl = false) {
    const box = document.getElementById("hdl-code-box");
    let code = "";

    if (!isVhdl) {
      code += `// Generated by Logic Lab (Verilog-2001)\n`;
      code += `module logic_circuit (\n`;
      code += `  input wire clk,\n`;
      const switches = state.components.filter((c) => c.type === "switch" || c.type === "button");
      const leds = state.components.filter((c) => c.type === "led");

      switches.forEach((s, idx) => (code += `  input wire in_${idx},\n`));
      leds.forEach((l, idx) => (code += `  output wire out_${idx},\n`));
      code = code.replace(/,\n$/, "\n");
      code += `);\n\n`;

      code += `  // Gate instantiations\n`;
      state.components.forEach((c, idx) => {
        if (["and", "or", "not", "nand", "nor", "xor", "xnor"].includes(c.type)) {
          code += `  ${c.type} g_${idx} (net_out_${c.id}, net_in_${c.id}_0, net_in_${c.id}_1);\n`;
        }
      });
      code += `\nendmodule\n`;
    } else {
      code += `-- Generated by Logic Lab (VHDL 93/2008)\n`;
      code += `library IEEE;\nuse IEEE.STD_LOGIC_1164.ALL;\n\n`;
      code += `entity logic_circuit is\n  Port (\n`;
      code += `    clk : in STD_LOGIC;\n`;
      code += `    out_0 : out STD_LOGIC\n`;
      code += `  );\nend logic_circuit;\n\n`;
      code += `architecture Behavioral of logic_circuit is\nbegin\n`;
      code += `  -- Signal assignments\n`;
      code += `end Behavioral;\n`;
    }

    box.value = code;
  }

  document.getElementById("btn-hdl-verilog").addEventListener("click", () => {
    document.getElementById("btn-hdl-verilog").classList.add("active");
    document.getElementById("btn-hdl-vhdl").classList.remove("active");
    exportHDL(false);
  });

  document.getElementById("btn-hdl-vhdl").addEventListener("click", () => {
    document.getElementById("btn-hdl-vhdl").classList.add("active");
    document.getElementById("btn-hdl-verilog").classList.remove("active");
    exportHDL(true);
  });

  document.getElementById("btn-copy-hdl").addEventListener("click", () => {
    const box = document.getElementById("hdl-code-box");
    box.select();
    document.execCommand("copy");
    alert("HDL code copied to clipboard!");
  });

  // --- Presets Loader ---
  document.getElementById("btn-presets").addEventListener("click", () => {
    openModal("modal-presets");
  });

  document.querySelectorAll(".btn-load-preset").forEach((btn) => {
    btn.addEventListener("click", () => {
      loadPreset(btn.dataset.preset);
      closeModal("modal-presets");
    });
  });

  function loadPreset(preset) {
    saveSnapshot();
    state.components = [];
    state.wires = [];

    if (preset === "full_adder") {
      const swA = createComponent("switch", -120, -50);
      const swB = createComponent("switch", -120, 0);
      const swCin = createComponent("switch", -120, 50);
      const fa = createComponent("full_adder", 0, 0);
      const ledSum = createComponent("led", 120, -25);
      const ledCout = createComponent("led", 120, 25);
      state.components.push(swA, swB, swCin, fa, ledSum, ledCout);

      state.wires.push({ fromCompId: swA.id, fromPort: 0, toCompId: fa.id, toPort: 0, currentSignal: 0 });
      state.wires.push({ fromCompId: swB.id, fromPort: 0, toCompId: fa.id, toPort: 1, currentSignal: 0 });
      state.wires.push({ fromCompId: swCin.id, fromPort: 0, toCompId: fa.id, toPort: 2, currentSignal: 0 });
      state.wires.push({ fromCompId: fa.id, fromPort: 0, toCompId: ledSum.id, toPort: 0, currentSignal: 0 });
      state.wires.push({ fromCompId: fa.id, fromPort: 1, toCompId: ledCout.id, toPort: 0, currentSignal: 0 });
    } else if (preset === "sr_latch") {
      const swS = createComponent("switch", -120, -40);
      const swR = createComponent("switch", -120, 40);
      const nor1 = createComponent("nor", 0, -40);
      const nor2 = createComponent("nor", 0, 40);
      const ledQ = createComponent("led", 120, -40);
      const ledQBar = createComponent("led", 120, 40);
      state.components.push(swS, swR, nor1, nor2, ledQ, ledQBar);

      state.wires.push({ fromCompId: swR.id, fromPort: 0, toCompId: nor1.id, toPort: 0, currentSignal: 0 });
      state.wires.push({ fromCompId: swS.id, fromPort: 0, toCompId: nor2.id, toPort: 1, currentSignal: 0 });
      state.wires.push({ fromCompId: nor1.id, fromPort: 0, toCompId: nor2.id, toPort: 0, currentSignal: 0 });
      state.wires.push({ fromCompId: nor2.id, fromPort: 0, toCompId: nor1.id, toPort: 1, currentSignal: 0 });
      state.wires.push({ fromCompId: nor1.id, fromPort: 0, toCompId: ledQ.id, toPort: 0, currentSignal: 0 });
      state.wires.push({ fromCompId: nor2.id, fromPort: 0, toCompId: ledQBar.id, toPort: 0, currentSignal: 0 });
    } else if (preset === "counter4") {
      const clk = createComponent("clock", -140, -10);
      const cnt = createComponent("counter4", -20, 0);
      const hex = createComponent("hex_disp", 120, 0);
      state.components.push(clk, cnt, hex);

      state.wires.push({ fromCompId: clk.id, fromPort: 0, toCompId: cnt.id, toPort: 0, currentSignal: 0 });
      state.wires.push({ fromCompId: cnt.id, fromPort: 0, toCompId: hex.id, toPort: 0, currentSignal: 0 });
      state.wires.push({ fromCompId: cnt.id, fromPort: 1, toCompId: hex.id, toPort: 1, currentSignal: 0 });
      state.wires.push({ fromCompId: cnt.id, fromPort: 2, toCompId: hex.id, toPort: 2, currentSignal: 0 });
      state.wires.push({ fromCompId: cnt.id, fromPort: 3, toCompId: hex.id, toPort: 3, currentSignal: 0 });
    } else if (preset === "mux2") {
      const sw0 = createComponent("switch", -120, -50);
      const sw1 = createComponent("switch", -120, 0);
      const swSel = createComponent("switch", -120, 50);
      const mux = createComponent("mux2", 0, 0);
      const led = createComponent("led", 120, 0);
      state.components.push(sw0, sw1, swSel, mux, led);

      state.wires.push({ fromCompId: sw0.id, fromPort: 0, toCompId: mux.id, toPort: 0, currentSignal: 0 });
      state.wires.push({ fromCompId: sw1.id, fromPort: 0, toCompId: mux.id, toPort: 1, currentSignal: 0 });
      state.wires.push({ fromCompId: swSel.id, fromPort: 0, toCompId: mux.id, toPort: 2, currentSignal: 0 });
      state.wires.push({ fromCompId: mux.id, fromPort: 0, toCompId: led.id, toPort: 0, currentSignal: 0 });
    }

    state.camera.x = 0;
    state.camera.y = 0;
    state.camera.zoom = 1.0;
    settleSimulation();
    render();
  }

  // --- Initial Launch ---
  renderPalette("gates");
  resizeCanvas();
  loadPreset("full_adder");
  startSimTimer();

})();
