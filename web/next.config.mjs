/** @type {import('next').NextConfig} */
const nextConfig = {
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
