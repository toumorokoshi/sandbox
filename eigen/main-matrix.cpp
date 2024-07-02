#include <eigen3/Eigen/Dense>
#include <iostream>

int main()
{
    Eigen::Matrix<double, 3, 3> m;
    m << 1, 2, 3,
        4, 5, 6,
        7, 8, 9;
    Eigen::Ref<Eigen::Matrix<double, 1, 2>, 0, Eigen::InnerStride<3>> c = m.block<1, 2>(2, 0);
    // doesn't work, fails with:
    // /usr/local/include/eigen3/Eigen/src/Core/util/XprHelper.h:159:
    //   Eigen::internal::variable_if_dynamic<T, Value>::variable_if_dynamic(T)
    //     [with T = long int; int Value = 6]: Assertion `v == T(Value)' failed.
    c.setZero();
    std::cout << c << std::endl;
}
