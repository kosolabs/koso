import type { Meta, StoryObj } from "@storybook/svelte";
import TaskStatus from "./task-status.svelte";

const meta = {
  title: "Koso/TaskStatus",
  component: TaskStatus,
  tags: ["autodocs"],
} satisfies Meta<TaskStatus>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Main: Story = {
  args: {
    status: "In Progress",
  },
};
