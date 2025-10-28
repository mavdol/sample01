export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <div className="flex h-screen w-full pt-4 overflow-hidden">
      <div
        className="absolute top-0 left-0 h-7 w-full"
        data-tauri-drag-region
      ></div>
      {children}
    </div>
  );
}
