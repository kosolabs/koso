import Chip from "./chip.svelte";

type ChipProps = { title: string; description?: string };

function parseChipProps(name: string): ChipProps {
  const indexOfColon = name.indexOf(":");
  if (indexOfColon !== -1) {
    return {
      title: name.slice(0, indexOfColon).trim(),
      description: name,
    };
  }

  const indexOfSpace = name.indexOf(" ", 12);
  if (indexOfSpace !== -1) {
    return {
      title: name.slice(0, indexOfSpace).trim() + "...",
      description: name,
    };
  }

  return { title: name, description: "" };
}

export { Chip, parseChipProps, type ChipProps };