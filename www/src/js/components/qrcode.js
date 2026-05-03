// Minimal QR Code generator using Canvas API (no external libraries)
// Implements QR Version 1-4, Error Correction Level M
const QRCode = (() => {
  // Galois field arithmetic for Reed-Solomon
  const GF_EXP = new Uint8Array(512);
  const GF_LOG  = new Uint8Array(256);
  (() => {
    let x = 1;
    for (let i = 0; i < 255; i++) {
      GF_EXP[i] = x; GF_LOG[x] = i;
      x = x << 1;
      if (x & 0x100) x ^= 0x11D;
    }
    for (let i = 255; i < 512; i++) GF_EXP[i] = GF_EXP[i - 255];
  })();

  const gf_mul = (a, b) => a && b ? GF_EXP[(GF_LOG[a] + GF_LOG[b]) % 255] : 0;
  const gf_div = (a, b) => a ? GF_EXP[(GF_LOG[a] + 255 - GF_LOG[b]) % 255] : 0;

  function gf_poly_mul(p, q) {
    const r = new Uint8Array(p.length + q.length - 1);
    for (let j = 0; j < q.length; j++)
      for (let i = 0; i < p.length; i++)
        r[i + j] ^= gf_mul(p[i], q[j]);
    return r;
  }

  function gf_poly_div(dividend, divisor) {
    let msg = new Uint8Array(dividend);
    for (let i = 0; i < dividend.length - (divisor.length - 1); i++) {
      const coef = msg[i];
      if (!coef) continue;
      for (let j = 1; j < divisor.length; j++)
        if (divisor[j]) msg[i + j] ^= gf_mul(divisor[j], coef);
    }
    return msg.slice(-(divisor.length - 1));
  }

  function rs_generator(n) {
    let g = new Uint8Array([1]);
    for (let i = 0; i < n; i++) g = gf_poly_mul(g, new Uint8Array([1, GF_EXP[i]]));
    return g;
  }

  function rs_encode(data, n) {
    const gen = rs_generator(n);
    const padded = new Uint8Array(data.length + n);
    padded.set(data);
    return gf_poly_div(padded, gen);
  }

  // Encode text as byte mode QR
  function encode(text) {
    const bytes = new TextEncoder().encode(text);
    const len = bytes.length;

    // Version 1-4, Error Correction M
    // EC codewords: v1=10, v2=16, v3=26, v4=36
    const versions = [
      { v:1, cap:14,  size:21, ec:10, blocks:1 },
      { v:2, cap:26,  size:25, ec:16, blocks:1 },
      { v:3, cap:44,  size:29, ec:26, blocks:2 },
      { v:4, cap:64,  size:33, ec:36, blocks:2 },
    ];
    const ver = versions.find(v => v.cap >= len + 3) || versions[3];

    // Build bit stream
    const bits = [];
    const addBits = (val, n) => { for (let i = n-1; i >= 0; i--) bits.push((val >> i) & 1); };
    addBits(0b0100, 4);     // byte mode
    addBits(len, 8);        // character count
    for (const b of bytes) addBits(b, 8);
    // Terminator + padding
    addBits(0, 4);
    while (bits.length % 8) bits.push(0);
    const pads = [0xEC, 0x11];
    const totalDC = ver.size === 21 ? 19 : ver.size === 25 ? 34 : ver.size === 29 ? 55 : 80;
    let pi = 0;
    while (bits.length < totalDC * 8) { addBits(pads[pi++ & 1], 8); }

    // Data codewords
    const dc = new Uint8Array(totalDC);
    for (let i = 0; i < totalDC; i++) {
      for (let b = 0; b < 8; b++) dc[i] = (dc[i] << 1) | bits[i*8+b];
    }

    // RS error correction per block
    const ecPerBlock = Math.floor(ver.ec / ver.blocks);
    const combined = [];
    for (let bl = 0; bl < ver.blocks; bl++) {
      const blen = Math.floor(totalDC / ver.blocks);
      const bdata = dc.slice(bl * blen, (bl+1) * blen);
      const ec_ = rs_encode(bdata, ecPerBlock);
      combined.push(...bdata, ...ec_);
    }

    return { version: ver, data: new Uint8Array(combined) };
  }

  function buildMatrix(version, data) {
    const size = version.size;
    const M = Array.from({length: size}, () => new Int8Array(size).fill(-1));

    // Finder patterns + separators
    function finder(r, c) {
      for (let dr = 0; dr < 7; dr++)
        for (let dc = 0; dc < 7; dc++)
          M[r+dr][c+dc] = (dr===0||dr===6||dc===0||dc===6||
            (dr>=2&&dr<=4&&dc>=2&&dc<=4)) ? 1 : 0;
      // Separators — guard against out-of-bounds on small versions
      for (let i = 0; i < 8; i++) {
        if (r+7 < size && c+i < size) M[r+7][c+i] = 0;
        if (c+7 < size && r+i < size) M[r+i][c+7] = 0;
      }
    }
    finder(0,0); finder(0,size-7); finder(size-7,0);

    // Format info placeholder (mask 0)
    const fmt = 0b101010000010010; // format bits for EC=M, mask=0
    const fmtBits = [];
    for (let i = 14; i >= 0; i--) fmtBits.push((fmt >> i) & 1);
    // Top-left
    const pos = [0,1,2,3,4,5,7,8];
    for (let i = 0; i < 6; i++) { M[8][i] = fmtBits[i]; M[i][8] = fmtBits[14-i]; }
    M[8][7] = fmtBits[6]; M[7][8] = fmtBits[7]; M[8][8] = fmtBits[8];
    // Other corners
    for (let i = 0; i < 7; i++) {
      M[size-1-i][8] = fmtBits[i];
      M[8][size-8+i] = fmtBits[7+i];
    }
    M[size-8][8] = 1; // dark module

    // Timing
    for (let i = 8; i < size-8; i++) {
      M[6][i] = M[i][6] = i & 1 ? 0 : 1;
    }

    // Place data bits (zigzag, right to left)
    let bit = 0;
    const totalBits = data.length * 8;
    const placed = new Uint8Array(totalBits);
    // build flat bit array
    const allBits = [];
    for (const byte of data) for (let i = 7; i >= 0; i--) allBits.push((byte >> i) & 1);
    allBits.push(...new Array(64).fill(0)); // padding

    let col = size - 1;
    let up = true;
    let bitIdx = 0;
    while (col > 0) {
      if (col === 6) col--;
      for (let cnt = 0; cnt < size; cnt++) {
        const row = up ? size - 1 - cnt : cnt;
        for (let dc = 0; dc < 2; dc++) {
          const c = col - dc;
          if (M[row][c] === -1) {
            const bit = allBits[bitIdx++] || 0;
            // mask 0: (row + col) % 2 === 0
            M[row][c] = (row + c) % 2 === 0 ? bit ^ 1 : bit;
          }
        }
      }
      up = !up;
      col -= 2;
    }

    return M;
  }

  return {
    render(text, canvas, { size = 200 } = {}) {
      try {
        const { version, data } = encode(text);
        const M = buildMatrix(version, data);
        const n = version.size;
        const ctx = canvas.getContext('2d');
        const mod = Math.floor(size / (n + 8));
        const offset = Math.floor((size - mod * n) / 2);
        canvas.width = canvas.height = size;
        ctx.fillStyle = '#ffffff';
        ctx.fillRect(0, 0, size, size);
        ctx.fillStyle = '#000000';
        for (let r = 0; r < n; r++)
          for (let c = 0; c < n; c++)
            if (M[r][c]) ctx.fillRect(offset + c * mod, offset + r * mod, mod, mod);
      } catch(e) {
        const ctx = canvas.getContext('2d');
        ctx.fillStyle = '#888'; ctx.font = '12px sans-serif';
        ctx.fillText('QR error', 10, 20);
      }
    }
  };
})();
