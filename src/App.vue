<script setup>
import { usePetInteractions } from './config/usePetInteractions';
import Live2dViewer from "./views/Live2dViewer.vue";
import {ref} from "vue";

const {
  setIgnoreMouse, handleDrag, startRecording,
  stopRecording, handleAction
} = usePetInteractions();

let pressTimer = null;
let isRecording = false;
let startTimestamp = 0;
let startPos = { x: 0, y: 0 };
let isDraggingTriggered = ref(false); // 标记是否已经开启了系统拖拽
const isRecordingStatus = ref(false); // 新增：用于界面显示
const onDown = (e) => {
  if (e.button !== 0) return;

  startTimestamp = Date.now();
  startPos = { x: e.screenX, y: e.screenY };
  isRecording = false;
  isDraggingTriggered.value = false;

  // 1. 延迟判定录音：只有在没发生拖拽的情况下才开启录音
  pressTimer = setTimeout(() => {
    if (!isDraggingTriggered.value) {
      isRecording = true;
      startRecording();
    }
  }, 500);
};

// 监听移动：如果移动距离超过阈值，则视为拖拽
const onMove = (e) => {
  // if (isRecording || isDraggingTriggered.value || startPos.x === 0) return;
  //
  // const dx = e.screenX - startPos.x;
  // const dy = e.screenY - startPos.y;
  //
  // // 移动超过 5 像素则触发系统拖拽
  // if (Math.abs(dx) > 5 || Math.abs(dy) > 5) {
  //   isDraggingTriggered.value = true;
  //   clearTimeout(pressTimer); // 移动了就不录音了
  //   handleDrag(e);
  // }
  // 确保能获取到坐标，PIXI 事件通常在 e.data.global 或 nativeEvent 中
  const screenX = e.screenX || e.data?.global?.x;
  const screenY = e.screenY || e.data?.global?.y;

  if (isRecording || isDraggingTriggered.value || !startPos.x || !screenX) return;

  const dx = screenX - startPos.x;
  const dy = screenY - startPos.y;

  if (Math.abs(dx) > 5 || Math.abs(dy) > 5) {
    console.log("检测到移动，触发拖拽", dx, dy); // 添加日志确认是否进入此逻辑
    isDraggingTriggered.value = true;
    clearTimeout(pressTimer);
    handleDrag(e);
  }
};

const onUp = async () => {
  clearTimeout(pressTimer);
  const duration = Date.now() - startTimestamp;
  const finalStartTimestamp = startTimestamp;
  startTimestamp = 0; // 重置

  if (isDraggingTriggered.value) {
    // 如果已经触发了拖拽，系统会接管 mouseup，这里的逻辑可能不会执行
    return;
  }

  if (isRecording) {
    // 情况 A: 结束录音
    await stopRecording();
    isRecording = false;
  } else if (duration < 300 && duration > 0) {
    // 情况 B: 判定为快速点击
    handleAction();
  }
  await setIgnoreMouse(true)
};
const onPointerOver = () => {
  console.log("响应：关闭穿透");
  // setIgnoreMouse(false);
};

const onPointerOut = () => {
  // 延迟一小会儿再穿透，防止微小抖动导致的失焦
  setTimeout(() => {
    if (!isRecording && !isDraggingTriggered.value) {
      console.log("响应：开启穿透");
      // setIgnoreMouse(true);
    }
  }, 100);
};
</script>

<template>
  <div class="main-app">
    <Live2dViewer
        modelPath="model/runtime/kei_basic_free.model3.json"
        @pointerover="onPointerOver"
        @pointerout="onPointerOut"
        @pointerdown="onDown"
        @pointermove="onMove"
        @pointerup="onUp"
    />
  </div>
</template>

<style>
/* 彻底禁止点击时的所有高亮和轮廓 */
* {
  -webkit-tap-highlight-color: transparent;
  outline: none !important;
  user-select: none; /* 防止长按选中文本蓝色高亮 */
  margin: 0;
  padding: 0;
}
canvas {
  outline: none !important;
  border: none !important;
  box-shadow: none !important; /* 彻底移除这个该死的模糊阴影 */
  filter: none !important; /* 防止使用了 drop-shadow 滤镜 */
  border-radius: 0 !important; /* 清除可能存在的圆角 */
  background: transparent !important; /* 确保背景彻底透明 */
}
html, body, #app {
  margin: 0; padding: 0;
  width: 450px; height: 600px;
  background: transparent !important;
  overflow: hidden;
  /* 关键：PC端这里必须为 auto，否则JS永远收不到 hover 事件 */
  pointer-events: none; /* 全局穿透 */
}

.main-app { width: 100%; height: 100%; }
</style>