import type { Meta, StoryObj } from "@storybook/svelte";
import Navbar from "./navbar.svelte";

// More on how to set up stories at: https://storybook.js.org/docs/writing-stories
const meta = {
  title: "Koso/Navbar",
  component: Navbar,
  tags: ["autodocs"],
} satisfies Meta<Navbar>;

export default meta;
type Story = StoryObj<typeof meta>;

// More on writing stories with args: https://storybook.js.org/docs/writing-stories/args
export const Main: Story = {};
