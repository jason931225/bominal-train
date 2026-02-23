/** @type {import('next').NextConfig} */
function normalizedApiTarget(rawValue) {
  const value = (rawValue ?? "").trim();
  if (!value) return "http://api:8000";
  return value.replace(/\/+$/, "").replace(/\/api$/, "");
}

const apiProxyTarget = normalizedApiTarget(process.env.API_SERVER_URL ?? process.env.NEXT_PUBLIC_API_BASE_URL);

const nextConfig = {
  output: "standalone",
  env: {
    APP_VERSION: process.env.APP_VERSION || "0.0.0",
    BUILD_VERSION: process.env.BUILD_VERSION || "dev",
  },
  async rewrites() {
    return [
      {
        source: "/api/:path*",
        destination: `${apiProxyTarget}/api/:path*`,
      },
    ];
  },
  async redirects() {
    return [
      {
        source: "/account",
        destination: "/settings/account",
        permanent: true,
      },
      {
        source: "/payment",
        destination: "/settings/payment",
        permanent: true,
      },
      {
        source: "/admin",
        destination: "/admin/maintenance",
        permanent: false,
      },
    ];
  },
};

export default nextConfig;
