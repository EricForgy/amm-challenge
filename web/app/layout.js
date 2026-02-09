import "./globals.css";

export const metadata = {
  title: "AMM Challenge",
  description: "Local web UI for AMM strategy validation and simulation."
};

export default function RootLayout({ children }) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}
