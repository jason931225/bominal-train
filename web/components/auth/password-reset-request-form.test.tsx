import React from "react";

import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { LocaleProvider } from "@/components/locale-provider";
import { PasswordResetRequestForm } from "@/components/auth/password-reset-request-form";

describe("PasswordResetRequestForm", () => {
  it("prefills email from initialEmail prop", () => {
    render(
      <LocaleProvider initialLocale="en">
        <PasswordResetRequestForm initialEmail="prefill@example.com" />
      </LocaleProvider>,
    );

    expect(screen.getByLabelText("Email")).toHaveValue("prefill@example.com");
  });
});
