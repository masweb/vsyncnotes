<script lang="ts" setup>
import { vaultCreate, vaultUnlock, vaultStatus } from '@/services/tauriApi'

const appStore = useAppStore()

const vaultExists = ref(false)
const password = ref('')
const confirmPassword = ref('')
const errorMsg = ref('')
const loading = ref(false)

onMounted(async () => {
  const status = await vaultStatus()
  vaultExists.value = status.exists
})

const handleSubmit = async () => {
  errorMsg.value = ''
  if (!vaultExists.value && password.value !== confirmPassword.value) {
    errorMsg.value = 'Las contraseñas no coinciden'
    return
  }
  loading.value = true
  try {
    if (!vaultExists.value) {
      await vaultCreate(password.value)
    }
    await vaultUnlock(password.value)
    appStore.setView('main')
  } catch (e) {
    errorMsg.value = String(e)
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <div class="d-flex align-items-center justify-content-center h-100">
    <div style="width: 320px">
      <h5 class="mb-4">{{ vaultExists ? 'Desbloquear vault' : 'Crear vault' }}</h5>

      <div class="mb-3">
        <input
          v-model="password"
          type="password"
          class="form-control form-control-sm"
          placeholder="Contraseña"
          :disabled="loading"
          @keyup.enter="handleSubmit"
        />
      </div>

      <div v-if="!vaultExists" class="mb-3">
        <input
          v-model="confirmPassword"
          type="password"
          class="form-control form-control-sm"
          placeholder="Confirmar contraseña"
          :disabled="loading"
          @keyup.enter="handleSubmit"
        />
      </div>

      <p v-if="errorMsg" class="text-danger small mb-2">{{ errorMsg }}</p>

      <button class="btn btn-sm btn-primary w-100" :disabled="loading" @click="handleSubmit">
        {{ vaultExists ? 'Desbloquear' : 'Crear' }}
      </button>
    </div>
  </div>
</template>
