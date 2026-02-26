'use client'

import { createContext, useContext, useState, useEffect, ReactNode } from 'react'
import { update_step } from '@/actions/update_step'
import { StepsEav } from '@/types'

type SignUpContextType = {
  step: number;
  setStep: (newStep: number) => Promise<void>;
};

const SignUpContext = createContext<SignUpContextType | null>(null);

export const SignUpProvider = ({
  children,
  initialStep = 0, // Accept initialStep as a prop
}: {
  children: ReactNode;
  initialStep?: number;
}) => {
  const [step, setStepState] = useState(initialStep);

  useEffect(() => {
    // In case the initial step changes from the parent, update it
    setStepState(initialStep);
  }, [initialStep]);

  const setStep = async (newStep: number) => {
    try {
      // Call server action or logic to persist step on the server
      await update_step(newStep); // Update this with your server action
      setStepState(newStep);
    } catch (error) {
      console.error("Failed to update step:", error);
    }
  };

  return (
    <SignUpContext.Provider value={{ step, setStep }}>
      {children}
    </SignUpContext.Provider>
  );
};

export const useSignUpContext = () => {
  const context = useContext(SignUpContext);
  console.log('context: '+ JSON.stringify(context))
  if (context === null) {
    throw new Error("useSignUpContext must be used within a SignUpProvider");
  }
  return context;
}


export function useStepsOnClient(): StepsEav {
  const { step, setStep } = useSignUpContext()

  if (step === null) {
      return {data: null, action: null, error: {code: 404, message: 'unknown error with lang'}}
  }

  return {data: step, action: setStep, error: null}
}