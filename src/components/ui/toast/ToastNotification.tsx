import { X, CheckCircle2, AlertCircle, XCircle } from "lucide-react";
import toast from "react-hot-toast";

export type ToastType = "success" | "error" | "info";

interface ToastNotificationProps {
  type: ToastType;
  title: string;
  message?: string;
  toastId: string;
}

export const ToastNotification = ({
  type,
  title,
  message,
  toastId,
}: ToastNotificationProps) => {
  const getIcon = () => {
    switch (type) {
      case "success":
        return <CheckCircle2 size={20} strokeWidth={1.5} />;
      case "error":
        return <XCircle size={20} strokeWidth={1.5} />;
      case "info":
        return <AlertCircle size={20} strokeWidth={1.5} />;
    }
  };

  return (
    <div className={`toast-notification toast-notification--${type}`}>
      <div className="toast-notification__icon">{getIcon()}</div>
      <div className="toast-notification__content">
        <div className="toast-notification__title">{title}</div>
        {message && (
          <div className="toast-notification__message">{message}</div>
        )}
      </div>
      <button
        className="toast-notification__close"
        onClick={() => toast.dismiss(toastId)}
        aria-label="Close notification"
      >
        <X size={16} strokeWidth={1.5} />
      </button>
    </div>
  );
};
