import Link from "@risc0/ui/link";

export default function Footer() {
  return (
    <div className="space-x-2 py-6 pt-6 text-center text-muted-foreground text-xs">
      <span>
        Built by{" "}
        <Link href="https://www.risczero.com" target="_blank">
          Risc Zero
        </Link>
      </span>
      <span>•</span>
      <Link target="_blank" href="https://dev.risczero.com/api/">
        Docs
      </Link>
      <span>•</span>
      <Link target="_blank" href="https://github.com/risc0/risc0/">
        GitHub
      </Link>
    </div>
  );
}
