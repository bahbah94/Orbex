/**
 * Instrumentation file for Next.js
 * This runs once when the server starts up
 * Perfect for validating environment variables before the app runs
 */

export async function register() {
  // Only run on server-side
  if (process.env.NEXT_RUNTIME === "nodejs") {
    console.log("🔍 Validating environment variables...")

    try {
      // Import and validate env vars
      // This will throw an error if validation fails
      const { env, isProduction, isDevelopment } = await import("./lib/env")

      console.log("✅ Environment variables validated successfully")
      console.log(`📦 Running in ${env.NODE_ENV} mode`)

      if (isDevelopment) {
        console.log("🔧 Development mode - additional logging enabled")
      }

      if (isProduction) {
        console.log("🚀 Production mode - optimizations enabled")
      }

      // Log non-sensitive configuration
      console.log("⚙️  Configuration:")
      console.log(`  - Server URL: ${env.SERVER_URL}`)
    } catch (error) {
      console.error("❌ Environment validation failed:")
      console.error(error)

      if (process.env.NODE_ENV === "prod") {
        console.error("🛑 Exiting due to invalid environment configuration")
        process.exit(1)
      } else {
        console.warn("⚠️  Continuing in development mode with invalid config")
      }
    }
  }
}
