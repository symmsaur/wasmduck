use math;

use std::f64::consts::PI;
/// Wendland quintic kernel
pub fn kernel_2d(r: f64, h: f64) -> f64 {
    let inv_h = 1.0 / h;
    let q = r*inv_h;

    if q > 2.0 {
        return 0.0;
    }

    let alpha_d = 7.0 / 4.0 / PI * math::pow(inv_h, 2);
    return math::pow(1.0 - 0.5 * q, 4) * (2.0 * q + 1.0) * alpha_d;
}

/// Gradient of Wendland quintic kernel
pub fn grad_kernel_2d(x: f64, y: f64, h: f64) -> (f64, f64) {
    let r = math::length(x, y);
    let inv_h = 1.0 / h;
    let q = r*inv_h;

    if q > 2.0 {
        return (0.0, 0.0);
    }

    let alpha_d = 7.0 / 4.0 / PI * math::pow(inv_h, 2);
    let grad = alpha_d * 5.0 * math::pow(q - 2.0, 3)/(8.0 * h * h);
    return (grad * x, grad * y);
}

/// Laplacian of Wendland quintic kernel
pub fn laplace_kernel_2d(r: f64, h: f64) -> f64 {
    let inv_h = 1.0 / h;
    let q = r*inv_h;
    let alpha_d = 7.0 / 4.0 / PI * math::pow(inv_h, 2);

    return alpha_d * 5.0*(5.0 * q * q * q - 24.0 * q * q + 36.0 * q - 16.0)/(8.0 * h * h);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kernel_2d() {
        let tolerance = 1e-15;
        assert!((kernel_2d(0.5, 0.5)-0.4177817256162253).abs() < tolerance);
        assert!((kernel_2d(1.0, 0.5)-0.0000000000000000).abs() < tolerance);
        assert!((kernel_2d(0.0, 1.0)-0.5570423008216338).abs() < tolerance);
        assert!((kernel_2d(1.0, 1.0)-0.1044454314040563).abs() < tolerance);
        assert!((kernel_2d(2.0, 1.0)-0.0000000000000000).abs() < tolerance);
    }

    #[test]
    fn test_grad_kernel_2d() {
        let tolerance = 1e-15;
        let (gx, gy) = grad_kernel_2d(1., 2., 10.);
        assert!((gx - (-0.000195157614586787)).abs() < tolerance);
        assert!((gy - (-0.000390315229173573)).abs() < tolerance);
    }

    #[test]
    fn test_laplacian_kernel_2d() {
        let tolerance = 1e-15;
        let laplacian = laplace_kernel_2d(f64::sqrt(5.0), 10.);
        assert!((laplacian - (-0.000316617746208086)).abs() < tolerance);
    }
}