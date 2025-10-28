import toast from "react-hot-toast";
import { ToastNotification, ToastType } from "@/components/ui/toast";

interface ShowToastOptions {
  type: ToastType;
  title: string;
  message?: string;
  duration?: number;
}

export const showToast = ({
  type,
  title,
  message,
  duration = Infinity,
}: ShowToastOptions): string => {
  return toast.custom(
    (t) => (
      <ToastNotification
        type={type}
        title={title}
        message={message}
        toastId={t.id}
      />
    ),
    {
      duration,
      position: "top-right",
    }
  );
};

export const showSuccessToast = (
  title: string,
  message?: string,
  duration?: number
) => {
  return showToast({ type: "success", title, message, duration });
};

export const showErrorToast = (
  title: string,
  message?: string,
  duration?: number
) => {
  return showToast({ type: "error", title, message, duration });
};

export const showInfoToast = (
  title: string,
  message?: string,
  duration?: number
) => {
  return showToast({ type: "info", title, message, duration });
};
