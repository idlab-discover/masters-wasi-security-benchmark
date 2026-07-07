import argparse
import os
import cbor2
import matplotlib.pyplot as plt

def main():
    parser = argparse.ArgumentParser(description="Generate a boxplot of times from CBOR data files with linear scale.")
    parser.add_argument("files", nargs="+", help="List of CBOR files to process")
    parser.add_argument("--save", help="Optional filename to save the plot (e.g., output.png)", default=None)
    parser.add_argument("--title-extra", help="Optional text appended to the plot title", default="")
    args = parser.parse_args()

    data_to_plot = []
    labels = []

    for filepath in args.files:
        # Derive label from the parent directory and containing directory.
        # Example: .../blocking/allow/measurement.cbor -> blocking/allow
        normalized_path = os.path.normpath(filepath)
        containing_dir = os.path.dirname(normalized_path)
        parent_dir = os.path.dirname(containing_dir)
        parent_name = os.path.basename(parent_dir) if parent_dir else ""
        containing_name = os.path.basename(containing_dir) if containing_dir else ""
        # label = f"{parent_name}/{containing_name}" if parent_name else containing_name
        label = containing_name
        
        try:
            with open(filepath, 'rb') as f:
                data = cbor2.load(f)
            
            # Extract the appropriate array for the distribution plot
            if 'avg_values' in data:
                plot_data = data['avg_values']
            elif 'values' in data:
                plot_data = data['values']
            else:
                print(f"Warning: Neither 'avg_values' nor 'values' found in {filepath}. Skipping.")
                continue
                
            data_to_plot.append(plot_data)
            labels.append(label)
            
        except Exception as e:
            print(f"Error reading {filepath}: {e}")

    if not data_to_plot:
        print("No valid data found to plot.")
        return

    fig, ax = plt.subplots(figsize=(10, max(4, 0.8 * len(labels))))

    # Create a horizontal boxplot with linear scale
    bp = ax.boxplot(data_to_plot, tick_labels=labels, patch_artist=True, vert=False)
    
    # Style the boxplot
    for patch in bp['boxes']:
        patch.set_facecolor('lightblue')
        patch.set_alpha(0.90)

    ax.set_xlabel("Time (ns)")
    ax.set_ylabel("Test Case (what arguments are being checked)")
    extra = f" - {args.title_extra}" if args.title_extra else ""
    ax.set_title(f"Time per Host Function Call{extra}")
    ax.grid(axis="x", linestyle="--", alpha=0.7)

    fig.tight_layout()

    # Save or show
    if args.save:
        plt.savefig(args.save)
        print(f"Plot saved to {args.save}")
    else:
        plt.show()

if __name__ == "__main__":
    main()