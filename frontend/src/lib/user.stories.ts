import type { Meta, StoryObj } from "@storybook/svelte";
import User from "./user.svelte";

const meta = {
  title: "Koso/User",
  component: User,
  tags: ["autodocs"],
} satisfies Meta<User>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Main: Story = {
  args: {
    user: {
      name: "Test User",
      email: "test@gmail.com",
      picture:
        "https://lh3.googleusercontent.com/a/ACg8ocIRfl1MJrdKF_V8e46SQijmFzs1JoEaQLogCsOEIYC-T2Hk2xcPKw=s96-c",
      exp: 0,
    },
  },
};
