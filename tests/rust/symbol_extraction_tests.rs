//! Comprehensive symbol extraction tests
//! Tests symbol extraction across all supported languages with edge cases

use fast_context::core::CoreAnalyzer;
use std::fs;
use tempfile::TempDir;

#[cfg(test)]
mod symbol_extraction_tests {
    use super::*;

    #[test]
    fn test_rust_symbol_extraction() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        let rust_content = r#"
            use std::collections::HashMap;
            
            // Constants
            const MAX_SIZE: usize = 1000;
            static GLOBAL_VAR: i32 = 42;
            
            // Type alias
            type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
            
            // Enum
            #[derive(Debug)]
            pub enum Status {
                Active,
                Inactive,
                Pending(String),
            }
            
            // Struct
            pub struct User {
                pub id: u64,
                pub name: String,
                status: Status,
            }
            
            // Trait
            pub trait Validate {
                fn is_valid(&self) -> bool;
            }
            
            // Implementation
            impl User {
                pub fn new(id: u64, name: String) -> Self {
                    Self {
                        id,
                        name,
                        status: Status::Active,
                    }
                }
                
                pub fn get_id(&self) -> u64 {
                    self.id
                }
            }
            
            impl Validate for User {
                fn is_valid(&self) -> bool {
                    !self.name.is_empty()
                }
            }
            
            // Functions
            pub fn create_user(name: &str) -> User {
                User::new(1, name.to_string())
            }
            
            fn private_helper() -> i32 {
                42
            }
            
            // Macro
            macro_rules! debug_print {
                ($($arg:tt)*) => {
                    println!("DEBUG: {}", format!($($arg)*));
                };
            }
            
            // Module
            pub mod utils {
                pub fn helper() -> String {
                    "helper".to_string()
                }
            }
        "#;

        let rust_file = temp_path.join("test.rs");
        fs::write(&rust_file, rust_content).unwrap();

        let analyzer = CoreAnalyzer::new(temp_path.to_string_lossy().to_string(), None, None);
        let result = analyzer.find_symbols_in_file(rust_file.to_string_lossy().to_string());
        
        assert!(result.is_ok());
        let symbols = result.unwrap();
        
        // Should find multiple types of symbols
        assert!(symbols.len() >= 10);
        
        // Check for specific symbol types
        let symbol_strings: Vec<String> = symbols.iter().map(|s| s.to_lowercase()).collect();
        assert!(symbol_strings.iter().any(|s| s.contains("user")));
        assert!(symbol_strings.iter().any(|s| s.contains("status")));
        assert!(symbol_strings.iter().any(|s| s.contains("validate")));
        assert!(symbol_strings.iter().any(|s| s.contains("create_user")));
    }

    #[test]
    fn test_javascript_symbol_extraction() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        let js_content = r#"
            // Constants
            const API_URL = 'https://api.example.com';
            const MAX_RETRIES = 3;
            
            // Variables
            let currentUser = null;
            var globalConfig = {};
            
            // Functions
            function fetchUser(id) {
                return fetch(`${API_URL}/users/${id}`);
            }
            
            const createUser = async (userData) => {
                try {
                    const response = await fetch(`${API_URL}/users`, {
                        method: 'POST',
                        body: JSON.stringify(userData)
                    });
                    return response.json();
                } catch (error) {
                    console.error('Error creating user:', error);
                    throw error;
                }
            };
            
            // Classes
            class UserManager {
                constructor(apiUrl) {
                    this.apiUrl = apiUrl;
                    this.cache = new Map();
                }
                
                async getUser(id) {
                    if (this.cache.has(id)) {
                        return this.cache.get(id);
                    }
                    
                    const user = await fetchUser(id);
                    this.cache.set(id, user);
                    return user;
                }
                
                clearCache() {
                    this.cache.clear();
                }
                
                static getInstance() {
                    if (!UserManager.instance) {
                        UserManager.instance = new UserManager(API_URL);
                    }
                    return UserManager.instance;
                }
            }
            
            // Object with methods
            const utils = {
                formatName(firstName, lastName) {
                    return `${firstName} ${lastName}`;
                },
                
                validateEmail(email) {
                    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
                    return emailRegex.test(email);
                }
            };
            
            // Export
            module.exports = {
                UserManager,
                fetchUser,
                createUser,
                utils
            };
        "#;

        let js_file = temp_path.join("test.js");
        fs::write(&js_file, js_content).unwrap();

        let analyzer = CoreAnalyzer::new(temp_path.to_string_lossy().to_string(), None, None);
        let result = analyzer.find_symbols_in_file(js_file.to_string_lossy().to_string());
        
        assert!(result.is_ok());
        let symbols = result.unwrap();
        
        // Should find multiple symbols
        assert!(symbols.len() >= 8);
        
        let symbol_strings: Vec<String> = symbols.iter().map(|s| s.to_lowercase()).collect();
        assert!(symbol_strings.iter().any(|s| s.contains("fetchuser") || s.contains("fetch_user")));
        assert!(symbol_strings.iter().any(|s| s.contains("usermanager") || s.contains("user_manager")));
        assert!(symbol_strings.iter().any(|s| s.contains("createuser") || s.contains("create_user")));
    }

    #[test]
    fn test_python_symbol_extraction() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        let python_content = r#"
            import os
            from typing import List, Optional, Dict
            
            # Constants
            MAX_CONNECTIONS = 100
            DEFAULT_TIMEOUT = 30
            
            # Global variables
            _connection_pool = None
            logger = None
            
            # Functions
            def connect_to_database(host: str, port: int = 5432) -> bool:
                """Connect to the database."""
                global _connection_pool
                try:
                    # Connection logic here
                    return True
                except Exception as e:
                    print(f"Connection failed: {e}")
                    return False
            
            async def fetch_users(limit: int = 10) -> List[Dict]:
                """Fetch users from the database."""
                # Async fetch logic
                return []
            
            def _private_helper(data: str) -> str:
                """Private helper function."""
                return data.strip().lower()
            
            # Classes
            class DatabaseManager:
                """Manages database connections and operations."""
                
                def __init__(self, host: str, port: int = 5432):
                    self.host = host
                    self.port = port
                    self._connected = False
                
                def connect(self) -> bool:
                    """Establish database connection."""
                    self._connected = connect_to_database(self.host, self.port)
                    return self._connected
                
                def disconnect(self) -> None:
                    """Close database connection."""
                    self._connected = False
                
                @property
                def is_connected(self) -> bool:
                    """Check if connected to database."""
                    return self._connected
                
                @staticmethod
                def get_default_config() -> Dict:
                    """Get default database configuration."""
                    return {
                        'host': 'localhost',
                        'port': 5432,
                        'timeout': DEFAULT_TIMEOUT
                    }
                
                @classmethod
                def from_config(cls, config: Dict) -> 'DatabaseManager':
                    """Create instance from configuration."""
                    return cls(config['host'], config['port'])
            
            class User:
                """User model class."""
                
                def __init__(self, id: int, name: str, email: str):
                    self.id = id
                    self.name = name
                    self.email = email
                
                def __str__(self) -> str:
                    return f"User({self.id}, {self.name})"
                
                def __repr__(self) -> str:
                    return f"User(id={self.id}, name='{self.name}', email='{self.email}')"
                
                def validate(self) -> bool:
                    """Validate user data."""
                    return bool(self.name and self.email and '@' in self.email)
            
            # Exception classes
            class DatabaseError(Exception):
                """Custom database exception."""
                pass
            
            class ConnectionError(DatabaseError):
                """Connection-specific exception."""
                pass
            
            # Decorator
            def retry(max_attempts: int = 3):
                """Retry decorator for functions."""
                def decorator(func):
                    def wrapper(*args, **kwargs):
                        for attempt in range(max_attempts):
                            try:
                                return func(*args, **kwargs)
                            except Exception as e:
                                if attempt == max_attempts - 1:
                                    raise e
                        return None
                    return wrapper
                return decorator
            
            # Module-level initialization
            if __name__ == "__main__":
                db = DatabaseManager.from_config(DatabaseManager.get_default_config())
                if db.connect():
                    print("Database connected successfully")
                else:
                    print("Failed to connect to database")
        "#;

        let python_file = temp_path.join("test.py");
        fs::write(&python_file, python_content).unwrap();

        let analyzer = CoreAnalyzer::new(temp_path.to_string_lossy().to_string(), None, None);
        let result = analyzer.find_symbols_in_file(python_file.to_string_lossy().to_string());
        
        assert!(result.is_ok());
        let symbols = result.unwrap();
        
        // Should find many symbols
        assert!(symbols.len() >= 15);
        
        let symbol_strings: Vec<String> = symbols.iter().map(|s| s.to_lowercase()).collect();
        assert!(symbol_strings.iter().any(|s| s.contains("databasemanager") || s.contains("database_manager")));
        assert!(symbol_strings.iter().any(|s| s.contains("user")));
        assert!(symbol_strings.iter().any(|s| s.contains("connect")));
        assert!(symbol_strings.iter().any(|s| s.contains("fetch")));
    }

    #[test]
    fn test_typescript_symbol_extraction() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        let ts_content = r#"
            // Type definitions
            type UserId = string;
            type UserRole = 'admin' | 'user' | 'guest';
            
            // Interfaces
            interface User {
                id: UserId;
                name: string;
                email: string;
                role: UserRole;
                createdAt: Date;
            }
            
            interface UserRepository {
                findById(id: UserId): Promise<User | null>;
                create(user: Omit<User, 'id' | 'createdAt'>): Promise<User>;
                update(id: UserId, updates: Partial<User>): Promise<User>;
                delete(id: UserId): Promise<void>;
            }
            
            // Enums
            enum Status {
                PENDING = 'pending',
                APPROVED = 'approved',
                REJECTED = 'rejected'
            }
            
            // Classes
            class UserService implements UserRepository {
                private users: Map<UserId, User> = new Map();
                
                constructor(private apiUrl: string) {}
                
                async findById(id: UserId): Promise<User | null> {
                    return this.users.get(id) || null;
                }
                
                async create(userData: Omit<User, 'id' | 'createdAt'>): Promise<User> {
                    const user: User = {
                        ...userData,
                        id: this.generateId(),
                        createdAt: new Date()
                    };
                    
                    this.users.set(user.id, user);
                    return user;
                }
                
                async update(id: UserId, updates: Partial<User>): Promise<User> {
                    const existingUser = await this.findById(id);
                    if (!existingUser) {
                        throw new Error(`User with id ${id} not found`);
                    }
                    
                    const updatedUser = { ...existingUser, ...updates };
                    this.users.set(id, updatedUser);
                    return updatedUser;
                }
                
                async delete(id: UserId): Promise<void> {
                    this.users.delete(id);
                }
                
                private generateId(): UserId {
                    return Math.random().toString(36).substr(2, 9);
                }
            }
            
            // Generic functions
            function createRepository<T extends { id: string }>(
                items: T[] = []
            ): Repository<T> {
                return new Repository(items);
            }
            
            // Arrow functions with types
            const validateUser = (user: User): boolean => {
                return !!(user.name && user.email && user.email.includes('@'));
            };
            
            const formatUserName = (user: User): string => {
                return `${user.name} (${user.role})`;
            };
            
            // Namespace
            namespace UserUtils {
                export function isAdmin(user: User): boolean {
                    return user.role === 'admin';
                }
                
                export function canEdit(user: User, targetUser: User): boolean {
                    return isAdmin(user) || user.id === targetUser.id;
                }
            }
            
            // Module exports
            export {
                User,
                UserRepository,
                UserService,
                Status,
                validateUser,
                formatUserName,
                UserUtils
            };
            
            export default UserService;
        "#;

        let ts_file = temp_path.join("test.ts");
        fs::write(&ts_file, ts_content).unwrap();

        let analyzer = CoreAnalyzer::new(temp_path.to_string_lossy().to_string(), None, None);
        let result = analyzer.find_symbols_in_file(ts_file.to_string_lossy().to_string());
        
        assert!(result.is_ok());
        let symbols = result.unwrap();
        
        // Should find many symbols including interfaces, classes, functions
        assert!(symbols.len() >= 10);
        
        let symbol_strings: Vec<String> = symbols.iter().map(|s| s.to_lowercase()).collect();
        assert!(symbol_strings.iter().any(|s| s.contains("user")));
        assert!(symbol_strings.iter().any(|s| s.contains("userservice") || s.contains("user_service")));
        assert!(symbol_strings.iter().any(|s| s.contains("status")));
    }

    #[test]
    fn test_java_symbol_extraction() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        let java_content = r#"
            package com.example.users;
            
            import java.util.*;
            import java.time.LocalDateTime;
            
            // Annotations
            @Entity
            @Table(name = "users")
            public class User {
                @Id
                @GeneratedValue(strategy = GenerationType.IDENTITY)
                private Long id;
                
                @Column(nullable = false)
                private String name;
                
                @Column(unique = true)
                private String email;
                
                @Enumerated(EnumType.STRING)
                private UserRole role;
                
                private LocalDateTime createdAt;
                
                // Constructors
                public User() {}
                
                public User(String name, String email, UserRole role) {
                    this.name = name;
                    this.email = email;
                    this.role = role;
                    this.createdAt = LocalDateTime.now();
                }
                
                // Getters and setters
                public Long getId() { return id; }
                public void setId(Long id) { this.id = id; }
                
                public String getName() { return name; }
                public void setName(String name) { this.name = name; }
                
                public String getEmail() { return email; }
                public void setEmail(String email) { this.email = email; }
                
                public UserRole getRole() { return role; }
                public void setRole(UserRole role) { this.role = role; }
                
                public LocalDateTime getCreatedAt() { return createdAt; }
                
                // Business methods
                public boolean isAdmin() {
                    return role == UserRole.ADMIN;
                }
                
                public boolean canEdit(User targetUser) {
                    return isAdmin() || this.id.equals(targetUser.getId());
                }
                
                @Override
                public String toString() {
                    return String.format("User{id=%d, name='%s', email='%s', role=%s}", 
                                       id, name, email, role);
                }
                
                @Override
                public boolean equals(Object obj) {
                    if (this == obj) return true;
                    if (obj == null || getClass() != obj.getClass()) return false;
                    User user = (User) obj;
                    return Objects.equals(id, user.id);
                }
                
                @Override
                public int hashCode() {
                    return Objects.hash(id);
                }
            }
            
            // Enum
            public enum UserRole {
                ADMIN("Administrator"),
                USER("Regular User"),
                GUEST("Guest User");
                
                private final String displayName;
                
                UserRole(String displayName) {
                    this.displayName = displayName;
                }
                
                public String getDisplayName() {
                    return displayName;
                }
            }
            
            // Interface
            public interface UserRepository {
                Optional<User> findById(Long id);
                List<User> findAll();
                User save(User user);
                void deleteById(Long id);
                List<User> findByRole(UserRole role);
            }
            
            // Service class
            @Service
            public class UserService {
                private final UserRepository userRepository;
                
                public UserService(UserRepository userRepository) {
                    this.userRepository = userRepository;
                }
                
                public User createUser(String name, String email, UserRole role) {
                    User user = new User(name, email, role);
                    return userRepository.save(user);
                }
                
                public Optional<User> getUserById(Long id) {
                    return userRepository.findById(id);
                }
                
                public List<User> getAllUsers() {
                    return userRepository.findAll();
                }
                
                public List<User> getAdminUsers() {
                    return userRepository.findByRole(UserRole.ADMIN);
                }
                
                public void deleteUser(Long id) {
                    userRepository.deleteById(id);
                }
            }
        "#;

        let java_file = temp_path.join("User.java");
        fs::write(&java_file, java_content).unwrap();

        let analyzer = CoreAnalyzer::new(temp_path.to_string_lossy().to_string(), None, None);
        let result = analyzer.find_symbols_in_file(java_file.to_string_lossy().to_string());
        
        assert!(result.is_ok());
        let symbols = result.unwrap();
        
        // Should find many symbols including classes, methods, enums
        assert!(symbols.len() >= 15);
        
        let symbol_strings: Vec<String> = symbols.iter().map(|s| s.to_lowercase()).collect();
        assert!(symbol_strings.iter().any(|s| s.contains("user")));
        assert!(symbol_strings.iter().any(|s| s.contains("userrole") || s.contains("user_role")));
        assert!(symbol_strings.iter().any(|s| s.contains("userservice") || s.contains("user_service")));
        assert!(symbol_strings.iter().any(|s| s.contains("userrepository") || s.contains("user_repository")));
    }

    #[test]
    fn test_edge_case_symbol_extraction() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Test with edge cases: very long names, special characters, etc.
        let edge_case_content = r#"
            // Very long function name
            fn this_is_a_very_long_function_name_that_might_cause_issues_with_parsing_or_memory_allocation() -> i32 {
                42
            }
            
            // Unicode in identifiers (if supported)
            fn test_函数() -> String {
                "unicode".to_string()
            }
            
            // Numbers in names
            fn function_123() -> i32 { 123 }
            fn function_456_with_numbers() -> i32 { 456 }
            
            // Underscores and special patterns
            fn __private_function__() {}
            fn _leading_underscore() {}
            fn trailing_underscore_() {}
            
            // Nested structures
            mod outer {
                pub mod inner {
                    pub fn deeply_nested_function() {}
                    
                    pub struct DeeplyNestedStruct {
                        field: i32,
                    }
                }
            }
            
            // Generic with complex bounds
            fn complex_generic<T, U, V>() 
            where 
                T: Clone + Send + Sync,
                U: std::fmt::Debug + std::fmt::Display,
                V: Iterator<Item = T>
            {
            }
            
            // Macro with complex patterns
            macro_rules! complex_macro {
                ($($name:ident: $type:ty),* $(,)?) => {
                    $(
                        fn $name() -> $type {
                            Default::default()
                        }
                    )*
                };
            }
            
            complex_macro! {
                test_i32: i32,
                test_string: String,
                test_bool: bool,
            }
        "#;

        let edge_case_file = temp_path.join("edge_cases.rs");
        fs::write(&edge_case_file, edge_case_content).unwrap();

        let analyzer = CoreAnalyzer::new(temp_path.to_string_lossy().to_string(), None, None);
        let result = analyzer.find_symbols_in_file(edge_case_file.to_string_lossy().to_string());
        
        assert!(result.is_ok());
        let symbols = result.unwrap();
        
        // Should handle edge cases gracefully and find symbols
        assert!(symbols.len() >= 5);
        
        // Should find the long function name
        let symbol_strings: Vec<String> = symbols.iter().map(|s| s.to_lowercase()).collect();
        assert!(symbol_strings.iter().any(|s| s.contains("very_long_function") || s.contains("long")));
    }
}
