<template>
  <div class="aurora-backdrop" aria-hidden="true">
    <div class="aurora-grid"></div>
    <span class="aurora-blob aurora-blob--a"></span>
    <span class="aurora-blob aurora-blob--b"></span>
    <span class="aurora-blob aurora-blob--c"></span>
    <span class="aurora-blob aurora-blob--d"></span>
    <span class="aurora-ring aurora-ring--a"></span>
    <span class="aurora-ring aurora-ring--b"></span>
  </div>
</template>

<style scoped>
.aurora-backdrop {
  position: fixed;
  inset: -20%;
  overflow: hidden;
  pointer-events: none;
  z-index: 0;
}

.aurora-grid {
  position: absolute;
  inset: 0;
  background-image:
    linear-gradient(rgba(148, 163, 184, 0.09) 1px, transparent 1px),
    linear-gradient(90deg, rgba(148, 163, 184, 0.09) 1px, transparent 1px);
  background-size: 120px 120px;
  mask-image: radial-gradient(circle at center, rgba(0, 0, 0, 0.85), transparent 85%);
  opacity: 0.65;
}

.aurora-blob,
.aurora-ring {
  position: absolute;
  border-radius: 999px;
  filter: blur(48px);
  opacity: 0.68;
  transform-origin: center;
}

.aurora-blob--a {
  width: 34rem;
  height: 34rem;
  left: 8%;
  top: 4%;
  background: radial-gradient(circle at 30% 30%, rgba(103, 232, 249, 0.95), rgba(14, 165, 233, 0.18) 58%, transparent 72%);
  animation: aurora-float-a 20s ease-in-out infinite;
}

.aurora-blob--b {
  width: 28rem;
  height: 28rem;
  right: 12%;
  top: 10%;
  background: radial-gradient(circle at 40% 35%, rgba(59, 130, 246, 0.92), rgba(14, 116, 144, 0.18) 54%, transparent 74%);
  animation: aurora-float-b 24s ease-in-out infinite;
}

.aurora-blob--c {
  width: 36rem;
  height: 36rem;
  left: 18%;
  bottom: 4%;
  background: radial-gradient(circle at 50% 50%, rgba(251, 191, 36, 0.78), rgba(249, 115, 22, 0.12) 62%, transparent 76%);
  animation: aurora-float-c 26s ease-in-out infinite;
}

.aurora-blob--d {
  width: 30rem;
  height: 30rem;
  right: 2%;
  bottom: 6%;
  background: radial-gradient(circle at 48% 42%, rgba(45, 212, 191, 0.84), rgba(56, 189, 248, 0.14) 58%, transparent 76%);
  animation: aurora-float-d 22s ease-in-out infinite;
}

.aurora-ring {
  border: 1px solid rgba(255, 255, 255, 0.38);
  background: radial-gradient(circle, rgba(255, 255, 255, 0.22), rgba(255, 255, 255, 0.03) 58%, transparent 72%);
}

.aurora-ring--a {
  width: 20rem;
  height: 20rem;
  left: 44%;
  top: 12%;
  animation: aurora-ring-a 18s ease-in-out infinite;
}

.aurora-ring--b {
  width: 14rem;
  height: 14rem;
  right: 16%;
  bottom: 24%;
  animation: aurora-ring-b 16s ease-in-out infinite;
}

@keyframes aurora-float-a {
  0%, 100% { transform: translate3d(0, 0, 0) scale(1); filter: blur(48px); }
  40% { transform: translate3d(7%, 12%, 0) scale(1.16); filter: blur(70px); }
  70% { transform: translate3d(10%, -2%, 0) scale(0.9); filter: blur(54px); }
}

@keyframes aurora-float-b {
  0%, 100% { transform: translate3d(0, 0, 0) scale(1); filter: blur(52px); }
  45% { transform: translate3d(-10%, 10%, 0) scale(1.18); filter: blur(74px); }
  72% { transform: translate3d(-4%, -8%, 0) scale(0.88); filter: blur(56px); }
}

@keyframes aurora-float-c {
  0%, 100% { transform: translate3d(0, 0, 0) scale(1); filter: blur(54px); }
  30% { transform: translate3d(-3%, -12%, 0) scale(1.12); filter: blur(72px); }
  65% { transform: translate3d(8%, -4%, 0) scale(0.92); filter: blur(60px); }
}

@keyframes aurora-float-d {
  0%, 100% { transform: translate3d(0, 0, 0) scale(1); filter: blur(50px); }
  38% { transform: translate3d(-12%, -8%, 0) scale(1.14); filter: blur(68px); }
  78% { transform: translate3d(0, 8%, 0) scale(0.9); filter: blur(58px); }
}

@keyframes aurora-ring-a {
  0%, 100% { transform: translate3d(0, 0, 0) scale(1); opacity: 0.52; }
  50% { transform: translate3d(-5%, 8%, 0) scale(1.18); opacity: 0.72; }
}

@keyframes aurora-ring-b {
  0%, 100% { transform: translate3d(0, 0, 0) scale(1); opacity: 0.34; }
  50% { transform: translate3d(10%, -10%, 0) scale(1.24); opacity: 0.58; }
}

@media (max-width: 768px) {
  .aurora-blob--a,
  .aurora-blob--c {
    width: 24rem;
    height: 24rem;
  }

  .aurora-blob--b,
  .aurora-blob--d {
    width: 20rem;
    height: 20rem;
  }
}
</style>
