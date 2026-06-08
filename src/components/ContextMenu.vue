<template>
  <Transition name="menu-fade">
    <div
      v-if="visible"
      class="context-menu-overlay"
      @click="closeMenu"
      @contextmenu.prevent
    >
      <div
        class="context-menu"
        :style="{ top: position.y + 'px', left: position.x + 'px' }"
        @click.stop
      >
        <div
          v-for="item in menuItems"
          :key="item.id"
          class="menu-item"
          :class="{ 'menu-divider': item.divider }"
          @click="handleItemClick(item)"
        >
          <template v-if="!item.divider">
            <span class="menu-icon">{{ item.icon }}</span>
            <span class="menu-label">{{ item.label }}</span>
          </template>
        </div>
      </div>
    </div>
  </Transition>
</template>

<script setup>
import { ref } from 'vue';

const visible = ref(false);
const position = ref({ x: 0, y: 0 });

const menuItems = [
  { id: 'weather', icon: '🌤️', label: '今天天气怎么样？' },
  { id: 'chat', icon: '💬', label: '陪我聊聊天' },
  { id: 'hide', icon: '👻', label: '隐藏宠物' },
  { id: 'divider', divider: true },
  { id: 'quit', icon: '✖️', label: '退出宠物' },
];

const emit = defineEmits(['menu-action']);

const show = (x, y) => {
  // 确保菜单不会超出屏幕边界
  const menuWidth = 200;
  const menuHeight = 220;
  const maxX = window.innerWidth - menuWidth;
  const maxY = window.innerHeight - menuHeight;

  position.value = {
    x: Math.max(0, Math.min(x, maxX)),
    y: Math.max(0, Math.min(y, maxY))
  };
  visible.value = true;
};

const closeMenu = () => {
  visible.value = false;
};

const handleItemClick = (item) => {
  if (item.divider) return;

  closeMenu();
  emit('menu-action', item.id);
};

defineExpose({ show, closeMenu });
</script>

<style scoped>
.context-menu-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  z-index: 9999;
  pointer-events: auto;
}

.context-menu {
  position: fixed;
  min-width: 180px;
  background: rgba(255, 255, 255, 0.95);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  border-radius: 12px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.15),
              0 2px 8px rgba(0, 0, 0, 0.1);
  padding: 8px;
  pointer-events: auto;
  overflow: hidden;
  border: 1px solid rgba(255, 255, 255, 0.8);
}

.menu-item {
  display: flex;
  align-items: center;
  padding: 10px 14px;
  cursor: pointer;
  border-radius: 8px;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  user-select: none;
  color: #333;
  font-size: 14px;
  gap: 10px;
}

.menu-item:not(.menu-divider):hover {
  background: linear-gradient(135deg,
    rgba(99, 102, 241, 0.1) 0%,
    rgba(168, 85, 247, 0.1) 100%);
  transform: translateX(4px);
}

.menu-item:not(.menu-divider):active {
  transform: translateX(4px) scale(0.98);
}

.menu-divider {
  height: 1px;
  background: linear-gradient(90deg,
    transparent 0%,
    rgba(0, 0, 0, 0.1) 50%,
    transparent 100%);
  margin: 6px 8px;
  padding: 0;
  cursor: default;
}

.menu-divider:hover {
  background: linear-gradient(90deg,
    transparent 0%,
    rgba(0, 0, 0, 0.1) 50%,
    transparent 100%);
  transform: none;
}

.menu-icon {
  font-size: 18px;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
}

.menu-label {
  flex: 1;
  font-weight: 500;
  letter-spacing: 0.3px;
}

/* 过渡动画 */
.menu-fade-enter-active,
.menu-fade-leave-active {
  transition: opacity 0.15s ease;
}

.menu-fade-enter-active .context-menu {
  animation: menuSlideIn 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}

.menu-fade-leave-active .context-menu {
  animation: menuSlideOut 0.15s cubic-bezier(0.4, 0, 0.2, 1);
}

.menu-fade-enter-from,
.menu-fade-leave-to {
  opacity: 0;
}

@keyframes menuSlideIn {
  from {
    opacity: 0;
    transform: scale(0.95) translateY(-5px);
  }
  to {
    opacity: 1;
    transform: scale(1) translateY(0);
  }
}

@keyframes menuSlideOut {
  from {
    opacity: 1;
    transform: scale(1) translateY(0);
  }
  to {
    opacity: 0;
    transform: scale(0.95) translateY(-5px);
  }
}
</style>
