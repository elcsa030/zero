import { Card, CardDescription, CardTitle } from "@risc0/ui/card";
import { Link } from "@risc0/ui/link";
import Image from "next/image";
import { REPORTS } from "./_utils/constants";

export default function ReportsPage() {
  return (
    <div className="container grid max-w-screen-xl grid-cols-1 gap-4 pt-4 lg:grid-cols-2">
      {REPORTS.map(({ label, href, description }, index) => (
        <Link key={href} href={href} className="group transition-opacity hover:opacity-70">
          <Card className="group-hover:-translate-y-1 flex h-full min-h-44 w-full flex-col items-center justify-between gap-1 px-8 py-4 shadow-sm transition-transform md:flex-row md:gap-12">
            <div>
              <CardTitle className="text-xl">{label}</CardTitle>
              <CardDescription className="text-sm">{description}</CardDescription>
            </div>
            <div className="flex min-h-[160px] min-w-[220px] justify-center">
              <Image
                width={220}
                height={160}
                priority
                className="user-select-none pointer-events-none hidden rounded dark:block"
                src={`/graph-${index}-dark.svg`}
                style={{
                  objectFit: "contain",
                  objectPosition: "right",
                }}
                alt={description}
              />
              <Image
                width={220}
                height={160}
                priority
                className={"user-select-none pointer-events-none rounded dark:hidden"}
                src={`/graph-${index}.svg`}
                style={{
                  objectFit: "contain",
                  objectPosition: "right",
                }}
                alt={description}
              />
            </div>
          </Card>
        </Link>
      ))}
    </div>
  );
}
