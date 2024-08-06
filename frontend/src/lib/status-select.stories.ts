import type { Meta, StoryObj } from "@storybook/svelte";
import StatusSelect from "./status-select.svelte";

const meta = {
  title: "Koso/StatusSelect",
  component: StatusSelect,
  tags: ["autodocs"],
} satisfies Meta<StatusSelect>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Main: Story = {
  args: {},
};
