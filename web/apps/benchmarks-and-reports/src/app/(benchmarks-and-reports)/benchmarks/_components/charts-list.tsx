import { Command, CommandEmpty, CommandGroup, CommandInput, CommandItem, CommandList } from "@risc0/ui/command";
import Link from "next/link";

export function ChartsList({ charts, selectedPlatform }) {
  if (!charts) {
    return null;
  }

  return (
    <Command className="border">
      <CommandInput placeholder={`${selectedPlatform} Benchmarks`} />
      <CommandList className="max-h-[calc(100dvh-19.5rem)] overscroll-contain">
        <CommandEmpty>No Results</CommandEmpty>
        {charts.flatMap((chart) =>
          // biome-ignore lint/correctness/useJsxKeyInIterable: no need for flatMap
          chart.name === selectedPlatform
            ? [
                <CommandGroup key={chart.name} heading={chart.name}>
                  {[...chart.dataSet.keys()].map((benchmark) => (
                    <Link key={`${chart.name}-${benchmark}`} scroll href={`#${chart.name}-${benchmark}`}>
                      <CommandItem className="cursor-pointer">{benchmark}</CommandItem>
                    </Link>
                  ))}
                </CommandGroup>,
              ]
            : [],
        )}
      </CommandList>
    </Command>
  );
}
