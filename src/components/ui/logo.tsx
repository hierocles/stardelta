"use client"

import logoWhite from "../../../assets/StarDelta Logo White.svg"

export function Logo() {
  return (
    <div className="flex justify-center">
      <img
        src={logoWhite}
        alt="StarDelta"
        className="h-12 w-auto"
      />
    </div>
  )
}
