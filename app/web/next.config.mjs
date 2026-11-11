// Üretimde: statik dışa aktarım (axum `app/web/out` dizinini sunar).
// Geliştirmede: `next dev -p 3001`; /api/* isteklerini axum'a (3000) yönlendirir.
// `output: 'export'` ile rewrites birlikte kullanılamadığı için export yalnızca üretimde açılır.
const dev = process.env.NODE_ENV === 'development';
const apiTarget = process.env.API_PROXY_TARGET ?? 'http://localhost:3000';

/** @type {import('next').NextConfig} */
const nextConfig = {
  trailingSlash: true,
  images: { unoptimized: true },
  reactStrictMode: true,
  agentRules: false,
  ...(dev
    ? {
        // trailingSlash: true iken /api/toc -> /api/toc/ yönlendirmesi API isteklerini bozar.
        skipTrailingSlashRedirect: true,
        async rewrites() {
          return [{ source: '/api/:path*', destination: `${apiTarget}/api/:path*` }];
        },
      }
    : { output: 'export' }),
};

export default nextConfig;
