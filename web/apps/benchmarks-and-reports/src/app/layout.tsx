import "@risc0/ui/styles/globals.css";
import "shared/styles/styles.css";

import { cn } from "@risc0/ui/cn";
import { JetBrains_Mono } from "next/font/google";
import type { PropsWithChildren } from "react";
import { Providers } from "shared/client/providers/providers";

export const metadata = {
  title: {
    template: "%s | RISC Zero Benchmarks & Reports",
    default: "RISC Zero Benchmarks & Reports",
  },
  metadataBase: new URL("https://benchmarks.risczero.com"),
  description: "Get to market fast with dramatically lower development costs on the first general purpose zkVM",
  openGraph: {
    images: [
      {
        url: "https://benchmarks.risczero.com/api/og?title=RISC%20Zero%20Benchmarks%20%26%20Reports", // @TODO: change with official URL
      },
    ],
  },
  icons: [
    {
      rel: "icon",
      url: "/favicon.png",
    },
  ],
};

const fontMono = JetBrains_Mono({
  subsets: ["latin"],
  variable: "--font-jetbrains-mono",
});

export default function RootLayout({ children }: PropsWithChildren) {
  return (
    <html lang="en" suppressHydrationWarning className="h-full">
      <body className={cn("flex min-h-full flex-col font-sans", fontMono.variable)}>
        <Providers>{children}</Providers>
      </body>
    </html>
  );
}
