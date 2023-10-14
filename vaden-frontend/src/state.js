import { reactive } from 'vue';

export const state = reactive({
  hello_world: {
    count: 0,
  },
});

export function increase() {
    state.hello_world.count += 1
    console.log('increased to ' + state.hello_world.count)
} 