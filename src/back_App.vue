<script setup>
import { onMounted, ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';

const canvasRef = ref();
const appWindow = getCurrentWindow();

// 调用 Rust 命令切换穿透状态
const setIgnoreMouse = (ignore) => {
  invoke('set_ignore_mouse', { ignore });
};

onMounted(() => {
  const initLive2D = async () => {
    // 检查 PIXI 是否加载
   const isReady = window.PIXI && window.PIXI.live2d && typeof window.PIXI.live2d.Live2DModel === 'function';
    if (!isReady) {
      setTimeout(initLive2D, 100);
      return;
    }

    const { Live2DModel } = window.PIXI.live2d;

    try {
      const app = new PIXI.Application({
        view: canvasRef.value,
        autoStart: true,
        backgroundAlpha: 0, // 必须透明
        width: 450,
        height: 600,
        antialias: true,
        // 显式设置事件模式
        eventMode: 'static',
      });

      const modelUrl = "model/runtime/kei_basic_free.model3.json";
      // autoInteract 设为 false 以便手动精准控制
      const model = await Live2DModel.from(modelUrl, {autoInteract: false});

      // 适配模型
      model.width = 400;
      model.scale.y = model.scale.x;
      model.x = 25;
      model.y = 600 - model.height - 20;

      // --- 关键交互逻辑 ---
      model.interactive = true;

      // 1. 实现鼠标悬停在人物身上时可点，悬停在空白处穿透
      model.on('pointerover', () => setIgnoreMouse(false)); // 进入人物，不穿透
      model.on('pointerout', () => setIgnoreMouse(true));   // 离开人物，穿透

      // 2. 实现鼠标左键按住人物拖动窗口
      model.on('pointerdown', (e) => {
        // 0 代表左键
        if (e.button === 0) {
          appWindow.startDragging();
        }
      });

      app.stage.addChild(model);

      // 初始设为穿透，直到检测到鼠标进入人物
      setIgnoreMouse(true);

    } catch (error) {
      console.error("PIXI 加载失败:", error);
    }
  };

  initLive2D();
});
</script>

<template>
  <div class="pet-wrapper">
    <canvas ref="canvasRef" class="pet-canvas"></canvas>
  </div>
</template>

<style>
/* 清除所有默认边距和背景 */
html, body, #app {
  margin: 0 !important;
  padding: 0 !important;
  width: 450px;
  height: 600px;
  overflow: hidden;
  background: transparent !important;
  user-select: none;
  /* 基础穿透 */
  pointer-events: none;
}

.pet-wrapper {
  width: 450px;
  height: 600px;
  position: relative;
}

.pet-canvas {
  width: 450px;
  height: 600px;
  display: block;
  /* 允许 Canvas 响应鼠标，之后配合 Rust 命令做更细致的穿透控制 */
  pointer-events: auto;
}
</style>