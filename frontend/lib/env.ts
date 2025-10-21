import { z } from "zod"

/**
 * Environment variable schema definition
 * Server-side variables (no NEXT_PUBLIC_ prefix) are only available on the server
 * Client-side variables (NEXT_PUBLIC_ prefix) are available on both server and client
 */
const envSchema = z.object({
  APP_ENV: z.enum(['dev', 'prod']).default('dev'),
  SERVER_URL: z.string().url("SERVER_URL must be a valid URL"),
})

/**
 * Type-safe environment variables
 */
export type Env = z.infer<typeof envSchema>

/**
 * Validates and returns environment variables
 * Throws an error if validation fails
 */
function validateEnv(): Env {
  const env = {
    APP_ENV: process.env.APP_ENV,
    SERVER_URL: process.env.SERVER_URL,
  }

  try {
    return envSchema.parse(env)
  } catch (error) {
    if (error instanceof z.ZodError) {
      const missingVars = error.errors.map((err) => {
        const path = err.path.join(".")
        return `  - ${path}: ${err.message}`
      })

      throw new Error(
        `❌ Invalid environment variables:\n${missingVars.join("\n")}\n\n` +
          "Please check your .env file and ensure all required variables are set.",
      )
    }
    throw error
  }
}

/**
 * Validated environment variables
 * Use this throughout your application instead of process.env
 */
export const env = validateEnv()
