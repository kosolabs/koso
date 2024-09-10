import type { Meta, StoryObj } from "@storybook/svelte";
import CircularProgress from "./circular-progress.svelte";

const meta = {
  title: "Koso/CircularProgress",
  component: CircularProgress,
  tags: ["autodocs"],
  parameters: {
    layout: "centered",
  },
} satisfies Meta<CircularProgress>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Main: Story = {
  args: {
    progress: 0.75,
  },
};
