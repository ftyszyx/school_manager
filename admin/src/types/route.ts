export enum RouteName {
    Home = 'home',
    Login = 'login',
    Register = 'register',
    Admin = 'admin',
    AdminDashboard = 'dashboard',
    AdminUsers = 'users',
    AdminRoles = 'roles',
    AdminSchools = 'schools',
    AdminClasses = 'classes',
    AdminPermissions = 'permissions',
}


export enum RoutePath {
    Home = '/',
    Login = '/' + RouteName.Login,
    Register = '/' + RouteName.Register,
    Admin = '/' + RouteName.Admin,
    AdminDashboard = '/' + RouteName.Admin + '/' + RouteName.AdminDashboard,
    AdminUsers = '/' + RouteName.Admin + '/' + RouteName.AdminUsers,
    AdminRoles = '/' + RouteName.Admin + '/' + RouteName.AdminRoles,
    AdminSchools = '/' + RouteName.Admin + '/' + RouteName.AdminSchools,
    AdminClasses = '/' + RouteName.Admin + '/' + RouteName.AdminClasses,
    AdminPermissions = '/' + RouteName.Admin + '/' + RouteName.AdminPermissions,
}
