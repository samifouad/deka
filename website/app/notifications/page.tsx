/* eslint-disable react/no-unescaped-entities */
/**
 * v0 by Vercel.
 * @see https://v0.dev/t/qB7S982mAHw
 * Documentation: https://v0.dev/docs#integrating-generated-code-into-your-nextjs-app
 */
import { Button } from "@/components/ui/button"

export default function NotificationsPage() {
  return (
    <div className="container mx-auto py-8 px-4 md:px-6">
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-2xl font-bold">Notifications</h1>
        <Button variant="outline">
          <CheckIcon className="mr-2 h-4 w-4" />
          Mark all as read
        </Button>
      </div>
      <div className="grid gap-6">
        <div>
          <h2 className="text-lg font-semibold mb-4">New</h2>
          <div className="grid gap-4">
            <div className="flex items-start gap-4">
              <div className="bg-gray-100 rounded-md flex items-center justify-center p-3 dark:bg-gray-800">
                <BellIcon className="h-5 w-5" />
              </div>
              <div className="flex-1">
                <h3 className="text-base font-medium">Your order has been shipped</h3>
                <p className="text-sm text-gray-500 dark:text-gray-400">
                  Your order #12345 has been shipped and is on its way to you.
                </p>
              </div>
            </div>
            <div className="flex items-start gap-4">
              <div className="bg-gray-100 rounded-md flex items-center justify-center p-3 dark:bg-gray-800">
                <MailboxIcon className="h-5 w-5" />
              </div>
              <div className="flex-1">
                <h3 className="text-base font-medium">New message from Jane</h3>
                <p className="text-sm text-gray-500 dark:text-gray-400">
                  Jane sent you a new message in the team chat.
                </p>
              </div>
            </div>
          </div>
        </div>
        <div>
          <h2 className="text-lg font-semibold mb-4">Unread</h2>
          <div className="grid gap-4">
            <div className="flex items-start gap-4">
              <div className="bg-gray-100 rounded-md flex items-center justify-center p-3 dark:bg-gray-800">
                <CalendarIcon className="h-5 w-5" />
              </div>
              <div className="flex-1">
                <h3 className="text-base font-medium">Upcoming meeting reminder</h3>
                <p className="text-sm text-gray-500 dark:text-gray-400">
                  You have a meeting scheduled for tomorrow at 2pm.
                </p>
              </div>
            </div>
            <div className="flex items-start gap-4">
              <div className="bg-gray-100 rounded-md flex items-center justify-center p-3 dark:bg-gray-800">
                <TicketIcon className="h-5 w-5" />
              </div>
              <div className="flex-1">
                <h3 className="text-base font-medium">Your support ticket has been updated</h3>
                <p className="text-sm text-gray-500 dark:text-gray-400">
                  Your support ticket #54321 has been updated with a new response.
                </p>
              </div>
            </div>
          </div>
        </div>
        <div>
          <h2 className="text-lg font-semibold mb-4">Read</h2>
          <div className="grid gap-4">
            <div className="flex items-start gap-4">
              <div className="bg-gray-100 rounded-md flex items-center justify-center p-3 dark:bg-gray-800">
                <GiftIcon className="h-5 w-5" />
              </div>
              <div className="flex-1">
                <h3 className="text-base font-medium">Your coupon has expired</h3>
                <p className="text-sm text-gray-500 dark:text-gray-400">
                  Your 20% off coupon for your next purchase has expired.
                </p>
              </div>
            </div>
            <div className="flex items-start gap-4">
              <div className="bg-gray-100 rounded-md flex items-center justify-center p-3 dark:bg-gray-800">
                <AwardIcon className="h-5 w-5" />
              </div>
              <div className="flex-1">
                <h3 className="text-base font-medium">You've earned a badge</h3>
                <p className="text-sm text-gray-500 dark:text-gray-400">
                  Congratulations, you've earned the "Frequent Flyer" badge for your 10th order.
                </p>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

function AwardIcon(props: any) {
  return (
    <svg
      {...props}
      xmlns="http://www.w3.org/2000/svg"
      width="24"
      height="24"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <circle cx="12" cy="8" r="6" />
      <path d="M15.477 12.89 17 22l-5-3-5 3 1.523-9.11" />
    </svg>
  )
}


function BellIcon(props: any) {
  return (
    <svg
      {...props}
      xmlns="http://www.w3.org/2000/svg"
      width="24"
      height="24"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <path d="M6 8a6 6 0 0 1 12 0c0 7 3 9 3 9H3s3-2 3-9" />
      <path d="M10.3 21a1.94 1.94 0 0 0 3.4 0" />
    </svg>
  )
}


function CalendarIcon(props: any) {
  return (
    <svg
      {...props}
      xmlns="http://www.w3.org/2000/svg"
      width="24"
      height="24"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <path d="M8 2v4" />
      <path d="M16 2v4" />
      <rect width="18" height="18" x="3" y="4" rx="2" />
      <path d="M3 10h18" />
    </svg>
  )
}


function CheckIcon(props: any) {
  return (
    <svg
      {...props}
      xmlns="http://www.w3.org/2000/svg"
      width="24"
      height="24"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <path d="M20 6 9 17l-5-5" />
    </svg>
  )
}


function GiftIcon(props: any) {
  return (
    <svg
      {...props}
      xmlns="http://www.w3.org/2000/svg"
      width="24"
      height="24"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <rect x="3" y="8" width="18" height="4" rx="1" />
      <path d="M12 8v13" />
      <path d="M19 12v7a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2v-7" />
      <path d="M7.5 8a2.5 2.5 0 0 1 0-5A4.8 8 0 0 1 12 8a4.8 8 0 0 1 4.5-5 2.5 2.5 0 0 1 0 5" />
    </svg>
  )
}


function MailboxIcon(props: any) {
  return (
    <svg
      {...props}
      xmlns="http://www.w3.org/2000/svg"
      width="24"
      height="24"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <path d="M22 17a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V9.5C2 7 4 5 6.5 5H18c2.2 0 4 1.8 4 4v8Z" />
      <polyline points="15,9 18,9 18,11" />
      <path d="M6.5 5C9 5 11 7 11 9.5V17a2 2 0 0 1-2 2v0" />
      <line x1="6" x2="7" y1="10" y2="10" />
    </svg>
  )
}


function TicketIcon(props: any) {
  return (
    <svg
      {...props}
      xmlns="http://www.w3.org/2000/svg"
      width="24"
      height="24"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <path d="M2 9a3 3 0 0 1 0 6v2a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-2a3 3 0 0 1 0-6V7a2 2 0 0 0-2-2H4a2 2 0 0 0-2 2Z" />
      <path d="M13 5v2" />
      <path d="M13 17v2" />
      <path d="M13 11v2" />
    </svg>
  )
}