<script setup>
import { onMounted, ref } from 'vue';

// 注意：这里不再 import PIXI 和 Live2DModel，完全从 window 获取
const canvasRef = ref();

onMounted(() => {
  const initLive2D = async () => {
// 更加精确的检查：直接检查构造函数是否存在
    const isReady = window.PIXI &&
        window.PIXI.live2d &&
        typeof window.PIXI.live2d.Live2DModel === 'function';

    console.log("检查 PIXI:", !!window.PIXI);
    console.log("检查 PIXI.live2d:", window.PIXI ? !!window.PIXI.live2d : false);
    console.log("检查 Live2DModel:", isReady);

    if (!isReady) {
      console.warn("Live2DModel 尚未就绪... 持续重试");
      setTimeout(initLive2D, 100);
      return;
    }

    // 只有当 isReady 为 true 时才执行后续逻辑
    const {Live2DModel} = window.PIXI.live2d;

    try {
      const app = new PIXI.Application({
        // 兼容性写法：优先尝试 view
        view: canvasRef.value,
        // 如果你后续升级到 v8，可以改成 canvas: canvasRef.value
        autoStart: true,
        backgroundAlpha: 0,
        resizeTo: window,
        antialias: true,
        hello: true, // 开启控制台欢迎语，确认 PIXI 运行正常
        // ✨ 强制开启事件处理，解决 currentTarget 报错
        eventMode: 'dynamic',
        eventFeatures: {
          move: true,
          globalMove: false,
          click: true,
          wheel: true,
        }
      });

      const modelUrl = "model/runtime/kei_basic_free.model3.json";
      const model = await Live2DModel.from(modelUrl, {autoInteract: false});

      // app.stage.addChild(model);
      // model.scale.set(0.2);
      // console.log("模型加载成功！");
      // ✨ 自适应缩放逻辑
      const fitModel = () => {
        const windowHeight = window.innerHeight;
        const windowWidth = window.innerWidth;

        // 1. 使用插件自带的比例适配方法
        // 这会将模型宽度或高度适配到屏幕的 80%
        const boundary = Math.min(windowWidth, windowHeight) * 0.8;

        // 这种方法会自动计算 scale，避开 coreModel.canvasHeight 可能为 undefined 的问题
        model.width = boundary;
        model.scale.y = model.scale.x; // 保持等比例缩放

        // 2. 居中定位
        model.x = (windowWidth - model.width) / 2;
        // model.y = windowHeight - model.height;
        // 向上偏移 20 像素，留出一点呼吸空间
        model.y = windowHeight - model.height - 20;

        console.log("当前缩放比例:", model.scale.x);
      };

      fitModel();
      app.stage.addChild(model);

// 监听窗口大小变化（比如手机横竖屏切换）
      window.addEventListener('resize', fitModel);

      // 在 model 加载成功后添加以下逻辑
      model.interactive = true; // 开启模型交互

    } catch (error) {
      console.error("加载过程中发生错误:", error);
    }
  };

  initLive2D();
});
</script>

<template>
  <canvas ref="canvasRef" class="pet-canvas"></canvas>
</template>

<style>
/* App.vue 中的样式 */
body, html, #app {
  margin: 0;
  padding: 0;
  width: 100vw;
  height: 100vh;
  overflow: hidden;
  /*  background: transparent !important; */
    /* 整个网页背景不接收点击，允许穿透到桌面图标 */
  background: rgba(255, 255, 255, 0.5) !important;
  pointer-events: none;
}

.pet-canvas {
  display: block;
  /* ✨ 核心：只有人物画布响应点击（如果你想点她的话） */
  pointer-events: none;
}
</style>