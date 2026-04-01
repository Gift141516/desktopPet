<script setup>
import { onMounted, ref } from 'vue';

const props = defineProps(['modelPath']);
const emit = defineEmits(['model-load', 'pointerover', 'pointerout', 'pointerdown', 'pointerup', 'pointermove']);
const canvasRef = ref();

const canvasWidth = 450;
const canvasHeight = 600;

onMounted(() => {
  const init = async () => {
    // const isReady = window.PIXI && window.PIXI.live2d && typeof window.PIXI.live2d.Live2DModel === 'function';
    //
    // if (!isReady) {
    if (!window.PIXI?.live2d?.Live2DModel) {
      setTimeout(init, 100);
      return;
    }
    const { Live2DModel } = window.PIXI.live2d;
    const app = new PIXI.Application({
      view: canvasRef.value,
      autoStart: true,
      backgroundAlpha: 0, // 必须透明
      width: 450,
      height: 600,
      antialias: true,
      autoInteract:false,
      // 显式设置事件模式
      eventMode: 'static',
    });

    // const model = await window.PIXI.live2d.Live2DModel.from(props.modelPath);
    //不能使用上面的方法,否则无法显示人物
    const model = await Live2DModel.from(props.modelPath, {autoInteract: false});
    const scaleX = canvasWidth / model.width;
    const scaleY = canvasHeight / model.height;
    const scale = Math.max(scaleX, scaleY);

    model.scale.set(scale);
    // 基础缩放和显示逻辑
    model.width = 600;
    model.scale.y = model.scale.x;
    // model.x = 25;
    // model.y = 10;
    model.x = (canvasWidth - model.width) / 2;
    model.y = (canvasHeight - model.height) / 2;
    model.interactive = true;
    // model.hitArea = new PIXI.Rectangle(0, 0, model.width / model.scale.x, model.height / model.scale.y);
    // model.hitArea = new PIXI.Ellipse(model.width/2, model.height/2, 150, 250);
    // 将事件暴露出去，不在此处写具体逻辑
    // model.on('pointerover', () => emit('pointerover'));
    model.on('pointerover', () => {
      console.log("鼠标进入人物范围");
      emit('pointerover');
    });
    // model.on('pointerout', () => emit('pointerout'));
    model.on('pointerout', () => {
      console.log("👈 鼠标离开人物范围");
      emit('pointerout');
    });
    model.on('pointerdown', (e) => emit('pointerdown', e));
    model.on('pointerup', (e) => emit('pointerup', e)); // 必须加上这一行
    model.on('pointermove', (e) => emit('pointermove', e));

    app.stage.addChild(model);
    emit('model-load', model);
  };
  init();
});
</script>

<template>
  <canvas ref="canvasRef" class="pet-canvas"></canvas>
</template>

<style scoped>
.pet-canvas { width: 100%; height: 100%; pointer-events: auto;box-sizing: border-box }

/* 强制对 Canvas 进行无残留清理 */
canvas, #app canvas, .pet-canvas {
  outline: none !important;
  border: none !important;
  box-shadow: none !important; /* 彻底移除这个该死的模糊阴影 */
  filter: none !important; /* 防止使用了 drop-shadow 滤镜 */
  border-radius: 0 !important; /* 清除可能存在的圆角 */
  background: transparent !important; /* 确保背景彻底透明 */
}
</style>