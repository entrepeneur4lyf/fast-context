// TypeScript-specific features test file
// This file tests if the TypeScript extractor properly handles TS-specific constructs

// Interface declaration
interface Calculator {
    add(a: number, b: number): number;
    subtract(a: number, b: number): number;
    multiply?(a: number, b: number): number; // Optional method
}

// Type alias
type Operation = 'add' | 'subtract' | 'multiply' | 'divide';
type NumberOrString = number | string;

// Generic interface
interface Repository<T> {
    save(entity: T): Promise<T>;
    findById(id: string): Promise<T | null>;
    findAll(): Promise<T[]>;
}

// Enum declaration
enum Color {
    Red = "red",
    Green = "green",
    Blue = "blue"
}

// Namespace declaration
namespace MathUtils {
    export function factorial(n: number): number {
        return n <= 1 ? 1 : n * factorial(n - 1);
    }
    
    export const PI = 3.14159;
    
    export interface Point {
        x: number;
        y: number;
    }
}

// Abstract class
abstract class Shape {
    abstract area(): number;
    abstract perimeter(): number;
    
    protected name: string;
    
    constructor(name: string) {
        this.name = name;
    }
    
    public getName(): string {
        return this.name;
    }
}

// Class with generics and implements
class Circle extends Shape implements Calculator {
    private radius: number;
    
    constructor(radius: number) {
        super("Circle");
        this.radius = radius;
    }
    
    public area(): number {
        return Math.PI * this.radius * this.radius;
    }
    
    public perimeter(): number {
        return 2 * Math.PI * this.radius;
    }
    
    // Implementing Calculator interface
    add(a: number, b: number): number {
        return a + b;
    }
    
    subtract(a: number, b: number): number {
        return a - b;
    }
}

// Function with type annotations (example for type system testing)
function _processData<T>(data: T[], processor: (item: T) => boolean): T[] {
    return data.filter(processor);
}

// Async function with type annotations (example for type system testing)
async function _fetchUserData(userId: string): Promise<{ id: string; name: string; email: string }> {
    // Simulated API call
    return {
        id: userId,
        name: "John Doe",
        email: "john@example.com"
    };
}

// Decorator (if supported)
function logged(target: any, propertyName: string, descriptor: PropertyDescriptor) {
    const method = descriptor.value;
    descriptor.value = function (...args: any[]) {
        console.log(`Calling ${propertyName} with args:`, args);
        return method.apply(this, args);
    };
}

// Class with decorators
class UserService {
    @logged
    public createUser(name: string, email: string): void {
        console.log(`Creating user: ${name} (${email})`);
    }
}

// Module declaration
declare module "external-library" {
    export function externalFunction(param: string): number;
    export interface ExternalInterface {
        prop: string;
    }
}

// Export statements
export { Calculator, Operation, Color, MathUtils, Shape, Circle };
export type { Repository, NumberOrString };
export default UserService;
