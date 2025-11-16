import { z } from "zod"

/**
 * Environment variable schema definition
 * Server-side variables (no NEXT_PUBLIC_ prefix) are only available on the server
 * Client-side variables (NEXT_PUBLIC_ prefix) are available on both server and client
 */
const envSchema = z.object({
  NEXT_PUBLIC_APP_ENV: z.enum(['dev', 'prod']).default('dev'),
  NEXT_PUBLIC_SERVER_URL: z.string().url("SERVER_URL must be a valid URL"),
  NEXT_PUBLIC_INDEXER_URL: z.string().url("INDEXER_URL must be a valid URL"),
  NEXT_PUBLIC_INDEXER_WS_URL: z.string().url("INDEXER_WS_URL must be a valid WebSocket URL"),
  // NEXT_PUBLIC_NODE_WS_URL: z.string().url("NODE_WS_URL must be a valid WebSocket URL"),
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
    NEXT_PUBLIC_APP_ENV: process.env.NEXT_PUBLIC_APP_ENV,
    NEXT_PUBLIC_SERVER_URL: process.env.NEXT_PUBLIC_SERVER_URL,
    NEXT_PUBLIC_INDEXER_URL: process.env.NEXT_PUBLIC_INDEXER_URL,
    NEXT_PUBLIC_INDEXER_WS_URL: process.env.NEXT_PUBLIC_INDEXER_WS_URL,
    // NEXT_PUBLIC_NODE_WS_URL: process.env.NEXT_PUBLIC_NODE_WS_URL,
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
 * Validated environment variables (raw with NEXT_PUBLIC_ prefix)
 * Use this throughout your application instead of process.env
 */
const rawEnv = validateEnv()

/**
 * Convenience exports with cleaner names
 * These are the actual values you should use in your app
 */
export const env = {
  APP_ENV: rawEnv.NEXT_PUBLIC_APP_ENV,
  SERVER_URL: rawEnv.NEXT_PUBLIC_SERVER_URL,
  INDEXER_URL: rawEnv.NEXT_PUBLIC_INDEXER_URL,
  INDEXER_WS_URL: rawEnv.NEXT_PUBLIC_INDEXER_WS_URL,
} as const
