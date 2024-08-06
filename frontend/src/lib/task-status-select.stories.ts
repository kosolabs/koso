import type { Meta, StoryObj } from "@storybook/svelte";
import TaskStatusSelect from "./task-status-select.svelte";

const meta = {
  title: "Koso/TaskStatusSelect",
  component: TaskStatusSelect,
  tags: ["autodocs"],
} satisfies Meta<TaskStatusSelect>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Main: Story = {
  args: {},
};
