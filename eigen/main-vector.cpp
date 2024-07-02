#include <eigen3/Eigen/Dense>
#include <iostream>

int main()
{
    Eigen::Vector<double, 1> m;
    m(1) = NAN;
    auto isFinite = m.allFinite();
    // works
    std::cout << "" << isFinite << std::endl;
}
