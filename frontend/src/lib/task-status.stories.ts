import type { Meta, StoryObj } from "@storybook/svelte";
import TaskStatustar from "./task-status.svelte";

const meta = {
  title: "Koso/TaskStatustar",
  component: TaskStatustar,
  tags: ["autodocs"],
} satisfies Meta<TaskStatustar>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Main: Story = {
  args: {
    status: "In Progress",
  },
};
