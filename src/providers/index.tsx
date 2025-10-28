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
        // containerStyle={{
        //   bottom: 24,
        //   right: 24,
        // }}
        toastOptions={{
          style: {
            background: "transparent",
            boxShadow: "none",
            padding: 0,
          },
        }}
      />
    </ThemeProvider>
  );
}
