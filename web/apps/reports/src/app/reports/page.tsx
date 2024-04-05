import Button from "@risc0/ui/button";
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@risc0/ui/card";
import Link from "@risc0/ui/link";
import Image from "next/image";

const REPORTS = [
  {
    label: "Benchmarks",
    href: "/reports/benchmarks",
    description: "View the latest benchmarks for the RISC Zero platform",
    cta: "View Benchmarks",
  },
  {
    label: "Datasheet",
    href: "/reports/datasheet",
    description: "View the latest datasheet for the RISC Zero platform",
    cta: "View Datasheet",
  },
  {
    label: "Applications Benchmarks",
    href: "/reports/applications-benchmarks",
    description: "View the latest applications benchmarks for the RISC Zero platform",
    cta: "View Applications Benchmarks",
  },
  {
    label: "Crates.io Validation",
    href: "/reports/crates-io-validation",
    description: "View the latest crates.io validation results",
    cta: "View Validation",
  },
] as const;

export default function ReportsPage() {
  return (
    <div className="container grid max-w-screen-md gap-4 pt-4 sm:grid-cols-2">
      {REPORTS.map(({ label, href, description, cta }, index) => (
        <Link key={href} href={href} className="group transition-opacity hover:opacity-70">
          <Card className="group-hover:-translate-y-1 flex h-full flex-col shadow-sm transition-transform">
            <CardHeader className="pb-4">
              <CardTitle className="text-2xl">{label}</CardTitle>
              <CardDescription>{description}</CardDescription>
            </CardHeader>
            <CardContent className="flex flex-1 items-end">
              <Image
                width={294}
                height={147}
                className="user-select-none pointer-events-none h-[147px] w-full rounded-sm object-cover shadow-xl"
                src={`/benchmarks-${index}.jpg`}
                alt={description}
              />
            </CardContent>
            <CardFooter>
              <Button tabIndex={-1} className="pointer-events-none w-full">
                {cta}
              </Button>
            </CardFooter>
          </Card>
        </Link>
      ))}
    </div>
  );
}
