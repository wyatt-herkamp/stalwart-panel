import type { Meta } from '@storybook/vue3'

import SubmitButton from './SubmitButton.vue'

const meta: Meta<typeof SubmitButton> = {
  component: SubmitButton,
  title: 'SubmitButton',
  tags: ['autodocs'],
  parameters: {
    docs: {
      description: {
        component: 'SubmitButton'
      }
    },
    slots: {
      default: {
        template: '{{ args.default || "SubmitButton" }}',
        description: 'button text'
      }
    }
  }
} satisfies Meta<typeof SubmitButton>

export default meta
export const Default = {
  args: {
    default: 'Submit'
  }
}

export const Danger = {
  args: {
    default: 'Danger',
    styleType: 'danger'
  }
}
export const Warning = {
  args: {
    default: 'Warning',
    styleType: 'warning'
  }
}
export const Disabled = {
  args: {
    default: 'Disabled',
    disabled: true
  }
}
