<template>
  <div class="flex flex-col h-full">
    <div class="text-2xl font-bold p-5 text-center">{{ $t('app.title') }}</div>
    <AppMenu :items="menuItems" mode="vertical" class="flex-1" />
  </div>
</template>

<script setup lang="ts">
import AppMenu from './AppMenu.vue'
import type { AdminMenuItem } from '@/types/menu'
import { RoutePath } from '@/types/route'



const menuItems: AdminMenuItem[] = [
  //i18n-key: menu.dashboard
  { label: 'menu.dashboard', icon: 'House', path: RoutePath.AdminDashboard },
  //i18n-key: menu.users
  { label: 'menu.users', icon: 'User', path: RoutePath.AdminUsers },
  //i18n-key: menu.roles
  { label: 'menu.roles', icon: 'UserFilled', path: RoutePath.AdminRoles },
  //i18n-key: menu.permissions
  { label: 'menu.permissions', icon: 'Lock', path: RoutePath.AdminPermissions },
  //i18n-key: menu.schools
  { label: 'menu.schools', icon: 'School', path: RoutePath.AdminSchools },
  //i18n-key: menu.classes
  { label: 'menu.classes', icon: 'Reading', path: RoutePath.AdminClasses },
]
</script>

<style scoped>
</style>


