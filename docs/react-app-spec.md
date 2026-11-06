# BioViva React Application Specification

## Executive Summary
React frontend application for the BioViva e-commerce platform, providing a responsive user interface for premium organic bio-certified products. The frontend consumes the Rust/Axum backend API and implements the user journeys defined in the PRD.

## 1. System Architecture & Integration

### 1.1 API Integration Layer
- **Base API URL**: `http://localhost:3000/api` (development) / `https://api.bioviva.example.com` (production)
- **Authentication**: JWT tokens stored in `httpOnly` cookies or `localStorage` with Axum-authenticated `Authorization: Bearer <token>` headers
- **Error Handling**: Centralized Axios error interceptor mapping to user-friendly messages
- **Timeout**: 30-second request timeout with automatic retry logic (max 2 retries for 5xx errors)

### 1.2 State Management
- **Global State**: React Context API with `useReducer` for auth, cart, and product state
- **Local State**: React `useState`/ `useReducer` for component-level UI state
- **Server State**: React Query (`@tanstack/react-query`) for caching, background refetching, and stale-while-revalidate patterns

### 1.3 Routing
- **Router**: React Router DOM v6+ with `createBrowserRouter`
- **Routes**:
  - `/` – Homepage with featured products
  - `/products` – Product listing with filters
  - `/products/[id]` – Product detail page
  - `/cart` – Shopping cart summary
  - `/checkout` – Checkout flow
  - `/orders/[id]` – Order details and tracking
  - `/auth/login` – Login page
  - `/auth/register` – Registration page
  - `/profile` – User profile (authenticated)
  - `/admin` – Admin dashboard (authenticated with role check)

## 2. Component Hierarchy & UI Library

### 2.1 UI Component Library
- **Base Library**: shadcn/ui components (built on Radix UI + Tailwind CSS)
- **Styling**: Tailwind CSS v3.4 with custom BioViva design tokens
- **Theme**: Light/dark mode support with `prefers-color-scheme` detection
- **Responsive**: Mobile-first breakpoint strategy (sm: 640px, md: 768px, lg: 1024px, xl: 1280px)

### 2.2 Core Component Categories

| Category | Key Components |
|---|---|
| **Layout** | `NavBar`, `Sidebar`, `Footer`, `Breadcrumbs`, `MainContent` |
| **Forms** | `LoginForm`, `RegisterForm`, `CheckoutForm`, `AddressForm`, `PasswordResetForm` |
| **Products** | `ProductCard`, `ProductGrid`, `ProductFilters`, `ProductDetail`, `VariantsSelector` |
| **Cart/Checkout** | `CartIcon`, `CartDrawer`, `CartItem`, `CheckoutSummary`, `OrderConfirmation` |
| **Authentication** | `LoginPage`, `RegisterPage`, `ProtectedRoute`, `AuthButton` |
| **Admin** | `AdminDashboard`, `ProductManager`, `OrderManager`, `UserManagement` |

### 2.3 Design Tokens (Tailwind Config)
```js
// tailwind.config.js
module.exports = {
  darkMode: ['class', 'system'],
  content: [
    './index.html',
    './src/**/*.{js,ts,jsx,tsx}',
  ],
  theme: {
    extend: {
      colors: {
        bioviva: {
          bg: '#F8F9FA',
          fg: '#1E293B',
          primary: '#0F172A',
          secondary: '#64748B',
          success: '#10B981',
          warning: '#F59E0B',
          error: '#EF4444',
        },
        organic: {
          green: '#059669',
          brown: '#78350F',
          parchment: '#F4E8D5',
        }
      },
      fontFamily: {
        sans: ['Inter', 'system-ui', 'sans-serif'],
        display: ['Playfair Display', 'serif'],
      },
      spacing: {
        1: '0.25rem',
        2: '0.5rem',
        // ... standard tailwind spacing
      }
    },
  },
  plugins: [require("tailwindcss-animate")],
}
```

## 3. Bounded Context Implementation

### 3.1 Auth Context (`src/contexts/AuthContext.tsx`)
- **State**: `user | null`, `isLoading`, `token`, `role`
- **Actions**: `login`, `logout`, `refreshToken`, `verifySession`
- **Protected Routes**: `ProtectedRoute` component checking `role` permissions
- **Token Management**: Automatic refresh on 401, token expiry tracking

### 3.2 Product Catalog Context (`src/contexts/ProductContext.tsx`)
- **State**: `products`, `filters`, `loading`, `error`, `categories`, `certifications`
- **Actions**: `fetchProducts`, `setFilters`, `setCategory`, `setCertification`
- **Query Logic**: Debounced search, filter state persistence, pagination

### 3.3 Cart Context (`src/contexts/CartContext.tsx`)
- **State**: `items`, `total`, `quantity`, `isCartOpen`
- **Actions**: `addItem`, `removeItem`, `updateQuantity`, `clearCart`, `calculateTotal`
- **Price Snapshotting**: Freeze price at add-time, match Rust backend `price_snapshot_eur` logic

### 3.4 Order Context (`src/contexts/OrderContext.tsx`)
- **State**: `orders`, `loading`, `error`, `selectedOrder`
- **Actions**: `fetchOrders`, `createOrder`, `cancelOrder`, `fetchOrderDetails`

## 4. API Service Layer

### 4.1 Axios Instance Configuration
```typescript
// src/services/api.ts
import axios from 'axios';
import { useContext } from 'react';
import { AuthContext } from '../contexts/AuthContext';

export const api = axios.create({
  baseURL: import.meta.env.VITE_API_URL || 'http://localhost:3000/api',
  timeout: 30000,
  headers: {
    'Content-Type': 'application/json',
  },
});

// Automatic auth header injection
api.interceptors.request.use((config) => {
  const { token } = useContext(AuthContext);
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

// Error interceptor
api.interceptors.response.use(
  (response) => response,
  (error) => {
    // Handle 401 - token expiry / refresh
    // Handle 403 - RBAC denial
    // Handle network errors
    return Promise.reject(error);
  }
);
```

### 4.2 Service Modules

| Service | Key Endpoints |
|---|---|
| `authService` | `POST /auth/register`, `POST /auth/login`, `POST /auth/refresh`, `POST /auth/logout` |
| `productService` | `GET /catalog/products`, `GET /catalog/products/[id]`, `GET /catalog/products/filters` |
| `cartService` | `POST /cart/add`, `POST /cart/remove`, `GET /cart`, `POST /cart/clear` |
| `orderService` | `POST /orders`, `GET /orders`, `GET /orders/[id]`, `POST /orders/[id]/cancel` |

## 5. Page Implementations

### 5.1 Homepage (`src/pages/HomePage.tsx`)
- Hero section with featured/rotating products
- Category grids with organic certification badges
- Search bar with debounced query
- "Shop by type" raw honey, olive oil, argan oil, herbal essences

### 5.2 Product Listing (`src/pages/ProductListPage.tsx`)
- Filter sidebar: organic standard, certifying body, acidity range, origin region, extraction method
- Product grid with lazy-loaded images
- Sort options: price low→high, price high→low, newest, best-rated
- Pagination with infinite scroll alternative

### 5.3 Product Detail (`src/pages/ProductDetailPage.tsx`)
- Product metadata: name, description, price, organic cert, extraction method, acidity, harvest date, origin
- Variant selector (size/weight) with price adjustment
- "Add to cart" with quantity input
- Batch traceability panel showing origin farm, harvest date, purity %, acidity %
- "Share" and "Wishlist" functionality

### 5.4 Cart & Checkout (`src/pages/CartPage.tsx` / `src/pages/CheckoutPage.tsx`)
- Cart item summary with snapshot prices
- Remove/quantity adjustment per item
- Checkout form: shipping address, billing address, payment method (placeholder for future)
- Order summary matching Rust backend state machine: `PendingPayment` → `Paid` → `Prepared` → `Shipped`

### 5.5 Auth Pages (`src/pages/AuthLoginPage.tsx` / `src/pages/AuthRegisterPage.tsx`)
- Registration: email, password (strength validation), confirm password
- Login: email, password, "remember me" remember token checkbox
- Social login placeholders (future Phase 2)

### 5.6 Admin Dashboard (`src/pages/AdminDashboard.tsx` - role-protected)
- Product CRUD management
- Order management with state transitions
- User management
- Batch/inventory reports
- System settings

## 6. Database Schema Alignment (Frontend → Backend)

### 6.1 Query Mapping
| Frontend Query | Backend SQL Schema | Example |
|---|---|---|
| `GET /catalog/products?standard=EU+Organic&min_acidity=0.3&max_acidity=0.8` | `catalog.products` | WHERE `certifying_body` = 'EU Organic' AND `acidity_range` BETWEEN '0.3' AND '0.8' |
| `GET /products/[id]` | `catalog.products` + `inventory.batches` | JOIN on `product_id` to show active batches |
| `GET /orders?state=Shipped` | `sales.orders` | WHERE `state` = 'Shipped' |
| `POST /cart/add?product_id=[id]&variant=[size]&quantity=[n]` | `sales.orders` + `inventory.batches` | Triggers checkout state machine |

### 6.2 State Synchronization
- Cart total must match `price_snapshot_eur` logic from Rust backend
- Order state transitions visible to user matching `sales.orders` state machine
- Batch stock numbers displayed matching `inventory.batches.stock_quantity`

## 7. Non-Functional Requirements (Frontend)

### 7.1 Performance
- **First Contentful Paint (FCP)**: < 1.5s on 3G
- **Time to Interactive (TTI)**: < 3s on 3G
- **API Response**: Handle loading, error, and skeleton states gracefully
- **Image Optimization**: Next.js/ImageOptim or Cloudinary integration for product photos

### 7.2 Accessibility (a11y)
- WCAG 2.1 AA compliance
- Keyboard navigation throughout
- Screen reader labels for all interactive elements
- Color contrast ratios ≥ 4.5:1 normal text, ≥ 3:1 large text

### 7.3 SEO
- Server-Side Rendering (SSR) with Next.js for product and collection pages
- Meta tags generation per page
- Structured data (JSON-LD) for products, organizations, breadcrumbs
- Sitemap.xml and robots.txt generation

### 7.4 Security (Frontend)
- JWT token validation on each request
- CSRF token protection (if using cookies)
- No sensitive data stored in localStorage for auth
- Secure HTTP-only cookies for token storage option

## 8. Development & Build Configuration

### 8.1 Tech Stack
- **Framework**: Next.js 14+ (App Router) – for SEO + SSR benefits
- **Language**: TypeScript 5.4+ (strict mode)
- **Package Manager**: pnpm
- **Styling**: Tailwind CSS v3.4
- **State**: React Query + Context API
- **Routing**: React Router DOM v6+
- **Forms**: React Hook Form + Zod validation
- **Icons**: Lucide React
- **API**: Axios with interceptors

### 8.2 Project Structure
```
src/
  app/                    # Next.js App Router
  components/            # Reusable UI components
    ui/                   # shadcn/ui components
    forms/                # Form components
    products/             # Product display components
    layout/               # Layout components
  contexts/               # React Context providers
  contexts/              # Context state
  lib/                    # Utility functions
  services/               # API service layer
  pages/                  # Page components (app router)
  types/                  # TypeScript type definitions
  hooks/                  # Custom React hooks
  styles/                 # Global styles
public/                   # Static assets
scripts/                  # Build/deployment scripts

### 8.3 Key Configuration Files
- `next.config.mjs` – Next.js configuration with TypeScript, Tailwind, env vars
- `tailwind.config.js` – Custom design tokens (see section 2.3)
- `tsconfig.json` – TypeScript strict mode, path aliases (@/*, ~/*)
- `src/env.d.ts` – Environment variable type definitions
- `.env.example` – Required variables: `VITE_API_URL`, `VITE_ANALYTICS_ID`

### 8.4 Build Commands
```bash
# Install
pnpm install

# Development
pnpm dev          # Next.js dev server with turbopack

# Build
pnpm build        # Production static build

# Analysis
pnpm analyse      # Bundle size analysis

# Lint/Format
pnpm lint         # ESLint + Prettier
pnpm format       # Code formatting
```

## 9. Deployment Pipeline

### 9.1 Environment Parity
- **Development**: `VITE_API_URL=http://localhost:3000/api` (Rust API via Docker)
- **Staging**: `VITE_API_URL=https://staging.api.bioviva.example.com`
- **Production**: `VITE_API_URL=https://api.bioviva.example.com`

### 9.2 CI/CD Integration
- **Build Artifact**: Next.js static exports or container image
- **Health Check**: `/api/health` endpoint reachability check
- **Feature Flags**: Environment-gated admin routes, experimental UI patterns

### 9.3 Rollback Strategy
- Deploy previous build version if `/health` endpoint fails post-deploy
- Database migration rollback separate from frontend deploy (Rust `make migrate-down`)

---

**Specification generated for BioViva React frontend integration. All components aligned with Rust backend API specifications, DDD bounded contexts, and quality gates defined in SPEC.md.**