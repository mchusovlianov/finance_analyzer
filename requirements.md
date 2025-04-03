# Detailed Transaction Viewer Requirements

## 1. Categorization Engine Details

### Pattern Matching Rules
- **Merchant-based rules**: Match transaction merchants against known entities
  ```rust
  // Example rule structure
  struct Rule {
      pattern: String,
      category: Category,
      priority: u8,
  }
  
  // Examples
  ["Albert Heijn", "Picnic", "Flink", "Crisp"] => Groceries
  ["ESSENT", "ANWB Energie", "Waternet"] => Utilities
  ```

### Rule Prioritization
- **Priority-based matching**: Higher priority rules take precedence when multiple rules match
- **Fallback categorization**: Default to "Uncategorized" when no rules match
- **Context-aware rules**: Consider transaction amount alongside merchant name
  ```
  If merchant contains "Albert Heijn" AND amount > 100 => "Major Grocery Shopping" 
  If merchant contains "Albert Heijn" AND amount < 20 => "Quick Grocery Run"
  ```

### Learning Capability
- Store manual category reassignments to improve future automatic categorization
- Build pattern database from repeated transactions

## 2. UI Flow and Interaction

### Views
1. **Main Transaction List View**
   - Sortable columns (date, amount, merchant, category)
   - Color-coded categories and transaction types (credit/debit)
   - Pagination for large datasets

2. **Category Summary View**
   - Bar chart showing spending by category
   - Income vs expense breakdown
   - Month-to-month comparison option

3. **Transaction Detail View**
   - Full transaction details including original description
   - Category assignment/reassignment
   - Notes field

### Navigation Map
```
Main List View <--> Category Summary View
     |                     |
     v                     v
Transaction Detail     Category Detail
```

### Key Bindings
- **Arrow keys**: Navigate lists
- **Tab**: Switch between main views
- **Enter**: Open transaction details
- **c**: Change category
- **f**: Filter transactions
- **s**: Sort transactions
- **q**: Quit application

## 3. Data Processing Pipeline

### Import Phase
1. **File reading**: Buffered reading of potentially large CSV files
2. **Validation**: Check for required fields and data integrity
3. **Normalization**: Convert dates and currency amounts to standard formats
4. **Initial categorization**: Apply categorization rules

### In-Memory Data Structure
```rust
// Application state
struct App {
    transactions: Vec<Transaction>,
    filtered_transactions: Vec<usize>, // Indices to transactions
    categories: HashMap<String, Category>,
    rules: Vec<Rule>,
    current_view: View,
    selected_transaction: Option<usize>,
    category_totals: HashMap<String, f64>,
}
```

### Filtering and Aggregation
- Lazy computation of aggregates
- Cached results for common filters
- Incremental updates when single transactions change categories

## 4. Error Handling Strategy

### CSV Parsing Errors
- Graceful handling of malformed lines
- Detailed error messages indicating line numbers and issues
- Option to skip problematic lines

### File Access Errors
- Clear messages for file not found or permission issues
- Suggestions for resolving common problems

### Runtime Error Recovery
- Crash prevention through proper error propagation
- Transaction log to recover state in case of unexpected termination

## 5. Testing Considerations

### Unit Tests
- Test categorization rules against sample transactions
- Test CSV parsing with various formats and edge cases

### Integration Tests
- End-to-end test of importing a file and viewing categories
- UI navigation testing

### Test Fixtures
- Sample CSV files with different formats and edge cases
- Known categorization examples for regression testing

## 6. Performance Optimizations

### Large File Handling
- Streaming parser that doesn't load entire file into memory
- Pagination of results
- Background processing of categorization

### UI Responsiveness
- Throttled redrawing
- Optimized rendering for visible elements only
- Background processing for expensive operations

### Memory Management
- Avoid string duplication for repeated fields like category names
- Use indices rather than cloning objects for filtered views

## 7. Additional Features for Future Consideration

### Reporting Capabilities
- Export to CSV/PDF
- Generate monthly spending reports
- Budget variance analysis

### Advanced Categorization
- Multi-level categorization (main category → subcategory)
- Regex pattern matching for complex descriptions
- Machine learning classification based on past data

### Persistent Storage
- SQLite backend for transaction storage
- Category definition persistence
- User preferences storage

Would you like me to elaborate on any specific aspect of these requirements further?