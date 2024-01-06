import type { Meta } from '@storybook/vue3'

import Switch from './Switch.vue'

const meta: Meta<typeof Switch> = {
  component: Switch,
  title: 'Switch',
  tags: ['autodocs'],
  parameters: {
    docs: {
      description: {
        component: 'Switch element'
      }
    },
    slots: {
      default: {
        template: '{{ args.default || "Switch" }}',
        description: 'button text'
      }
    }
  }
} satisfies Meta<typeof Switch>

export default meta
export const Default = {
  args: {
    id: 'switch',
    default: 'Submit'
  }
}

export const Disabled = {
  args: {
    default: 'Disabled',
    disabled: true
  }
}
