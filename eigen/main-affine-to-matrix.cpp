#include <eigen3/Eigen/Dense>
#include <iostream>

struct Transform3D
{
    /* Eigen affine 3D matrix: output = tf * point
            r00 r01 r02 tx
            r10 r11 r12 ty
            r20 r21 r22 tz
              0   0   0  1
    */
    float r00;
    float r01;
    float r02;
    float tx;
    float r10;
    float r11;
    float r12;
    float ty;
    float r20;
    float r21;
    float r22;
    float tz;
};

int main()
{
    Eigen::Affine3d tf = Eigen::Translation3d(-3.1f, 0.8f, 15.5f) * Eigen::AngleAxisd(-5.6, Eigen::Vector3d::UnitX()) * Eigen::AngleAxisd(11.0, Eigen::Vector3d::UnitY()) * Eigen::AngleAxisd(1.15, Eigen::Vector3d::UnitZ());

    const Eigen::Matrix4d matrix = tf.matrix();

    Transform3D as3D = {static_cast<float>(matrix(0, 0)),
                        static_cast<float>(matrix(0, 1)),
                        static_cast<float>(matrix(0, 2)),
                        static_cast<float>(matrix(0, 3)),
                        static_cast<float>(matrix(1, 0)),
                        static_cast<float>(matrix(1, 1)),
                        static_cast<float>(matrix(1, 2)),
                        static_cast<float>(matrix(1, 3)),
                        static_cast<float>(matrix(2, 0)),
                        static_cast<float>(matrix(2, 1)),
                        static_cast<float>(matrix(2, 2)),
                        static_cast<float>(matrix(2, 3))};
    std::cout << as3D.tz << std::endl;
    return 0;
}
