import { Space_Grotesk, IBM_Plex_Mono } from "next/font/google";
import "./globals.css";

const space = Space_Grotesk({
  subsets: ["latin"],
  variable: "--font-space"
});

const mono = IBM_Plex_Mono({
  weight: ["400", "500"],
  subsets: ["latin"],
  variable: "--font-mono"
});

export const metadata = {
  title: "AMM Challenge",
  description: "Local web UI for AMM strategy validation and simulation."
};

export default function RootLayout({ children }) {
  return (
    <html lang="en">
      <body className={`${space.variable} ${mono.variable}`}>{children}</body>
    </html>
  );
}
