export default defineEventHandler(async (event) => {
  return {
    success: true,
    message: 'API is working',
    timestamp: new Date().toISOString()
  }
})
