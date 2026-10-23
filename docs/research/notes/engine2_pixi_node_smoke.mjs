import { createCanvas, Image, ImageData, Path2D, DOMMatrix } from '@napi-rs/canvas';
import { DOMAdapter, autoDetectRenderer, Container, Graphics, Sprite, Texture, CanvasSource } from 'pixi.js';
import { writeFileSync } from 'node:fs';
const mk=(w,h)=>{const c=createCanvas(w??1,h??1); Object.assign(c,{style:{},addEventListener(){},removeEventListener(){},setAttribute(){},getBoundingClientRect:()=>({x:0,y:0,width:c.width,height:c.height,left:0,top:0}),parentNode:null}); return c;};
// Minimal globals the canvas renderer touches
globalThis.ImageData ??= ImageData; globalThis.Path2D ??= Path2D; globalThis.DOMMatrix ??= DOMMatrix; globalThis.Image ??= Image;
globalThis.HTMLCanvasElement ??= class {}; globalThis.HTMLImageElement ??= class {}; globalThis.HTMLVideoElement ??= class {};
globalThis.ImageBitmap ??= class {}; globalThis.OffscreenCanvas ??= class {};
globalThis.requestAnimationFrame ??= (cb) => setTimeout(() => cb(performance.now()), 16);
globalThis.cancelAnimationFrame ??= clearTimeout;
const el = () => ({ style: {}, remove(){}, appendChild(){}, removeChild(){}, setAttribute(){}, addEventListener(){}, removeEventListener(){}, getBoundingClientRect: () => ({x:0,y:0,width:0,height:0,left:0,top:0}), parentNode: null, children: [] });
globalThis.document ??= { createElement: (t) => t === 'canvas' ? mk(1,1) : el(), body: el(), documentElement: el(), addEventListener(){}, removeEventListener(){} };
globalThis.window ??= globalThis; globalThis.navigator ??= { userAgent: 'node', gpu: null };
globalThis.self ??= globalThis; globalThis.addEventListener ??= ()=>{}; globalThis.removeEventListener ??= ()=>{}; globalThis.location ??= { href: 'node://' };
DOMAdapter.set({
  createCanvas: (w, h) => mk(w, h),
  getCanvasRenderingContext2D: () => (createCanvas(1,1).getContext('2d')).constructor,
  getWebGLRenderingContext: () => class {},
  getNavigator: () => ({ userAgent: 'node', gpu: null }),
  getBaseUrl: () => 'node://',
  fetch: async () => { throw new Error('no fetch') },
  parseXML: () => { throw new Error('no xml') },
});
const t0 = performance.now();
const renderer = await autoDetectRenderer({ preference: 'canvas', width: 64, height: 48, background: 0x202020, antialias: false, roundPixels: true });
console.log('renderer type', renderer.name, 'init ms', (performance.now()-t0).toFixed(0));
const stage = new Container();
const src = createCanvas(2,2); const sc = src.getContext('2d'); sc.fillStyle='#ff0000'; sc.fillRect(0,0,1,1); sc.fillStyle='#0000ff'; sc.fillRect(1,1,1,1);
const tex = new Texture({ source: new CanvasSource({ resource: src, scaleMode: 'nearest', width: 2, height: 2 }) });
const spr = new Sprite(tex); spr.scale.set(8); spr.position.set(8, 8); stage.addChild(spr);
stage.addChild(new Graphics().rect(40, 8, 16, 16).fill(0x00ff00));
renderer.render({ container: stage });
const out = renderer.canvas; // napi canvas
const ctx = out.getContext('2d'); const px=(x,y)=>Array.from(ctx.getImageData(x,y,1,1).data);
console.log('px(11,11)=',px(11,11),'px(19,19)=',px(19,19),'px(44,12)=',px(44,12),'px(2,2)=',px(2,2));
writeFileSync('pixi-node.png', out.toBuffer('image/png'));
console.log('wrote pixi-node.png', out.width, out.height); process.exit(0);
