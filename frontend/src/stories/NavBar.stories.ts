import { NavBar } from "$lib/NavBar";
import type { Meta, StoryObj } from "@storybook/svelte";

// More on how to set up stories at: https://storybook.js.org/docs/writing-stories
const meta = {
  title: "Koso/NavBar",
  component: NavBar,
  tags: ["autodocs"],
} satisfies Meta<NavBar>;

export default meta;
type Story = StoryObj<typeof meta>;

// More on writing stories with args: https://storybook.js.org/docs/writing-stories/args
export const Main: Story = {};
