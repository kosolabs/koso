import type { Meta, StoryObj } from "@storybook/svelte";
import UserSelect from "./user-select.svelte";

const meta = {
  title: "Koso/UserSelect",
  component: UserSelect,
  tags: ["autodocs"],
  parameters: {
    layout: "centered",
  },
} satisfies Meta<UserSelect>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Main: Story = {
  args: {
    users: [
      {
        name: "Anvi",
        email: "anvi@gmail.com",
        picture:
          "https://lh3.googleusercontent.com/a/ACg8ocIv32sQsEfYy4G1xw-C-yYqRNEDSdBPXJME3hr5jc4kVu75mw1T=s96-c",
        exp: 0,
      },
      {
        name: "Kyle",
        email: "kyle@gmail.com",
        picture:
          "https://lh3.googleusercontent.com/a/ACg8ocIIqNHG-bPON1NKXNOCiJR8fCS_ze3iIAsCvunJ4_kyhKJXFA=s96-c",
        exp: 0,
      },
      {
        name: "Shad",
        email: "shad@gmail.com",
        picture:
          "https://lh3.googleusercontent.com/a/ACg8ocIRfl1MJrdKF_V8e46SQijmFzs1JoEaQLogCsOEIYC-T2Hk2xcPKw=s96-c",
        exp: 0,
      },
    ],
  },
};
