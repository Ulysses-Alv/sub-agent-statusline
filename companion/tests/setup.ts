// Vitest automatically provides 'expect' as a global
// This file sets up @testing-library/jest-dom matchers
import "@testing-library/jest-dom";

// If we need to extend manually, we would do:
// import { expect } from "vitest";
// import * as matchers from "@testing-library/jest-dom";
// expect.extend(matchers);