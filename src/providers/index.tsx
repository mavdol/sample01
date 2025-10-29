import { ThemeProvider } from "./theme.provider";
import { Toaster } from "react-hot-toast";

export default function CustomProviders({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <ThemeProvider>
      {children}
      <Toaster
        position="top-center"
        toastOptions={{
          style: {
            background: "transparent",
            boxShadow: "none",
            padding: 0,
          },
          removeDelay: 100,
        }}
      />
    </ThemeProvider>
  );
}
