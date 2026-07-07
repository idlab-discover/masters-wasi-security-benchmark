import argparse
import os
import statistics
import cbor2
import matplotlib.pyplot as plt

def main():
    parser = argparse.ArgumentParser(description="Generate a bar chart of average times from CBOR data files.")
    parser.add_argument("files", nargs="+", help="List of CBOR files to process")
    parser.add_argument("--save", help="Optional filename to save the plot (e.g., output.png)", default=None)
    parser.add_argument("--title-extra", help="Optional text appended to the plot title", default="")
    args = parser.parse_args()

    data_to_plot = []
    labels = []

    for filepath in args.files:
        # Derive label from the parent directory of the file's containing directory.
        # Example: .../blocking/allow/measurement.cbor -> blocking
        normalized_path = os.path.normpath(filepath)
        containing_dir = os.path.dirname(normalized_path)
        parent_dir = os.path.dirname(containing_dir)
        label = os.path.basename(parent_dir) if parent_dir else os.path.basename(containing_dir)
        
        try:
            with open(filepath, 'rb') as f:
                data = cbor2.load(f)
            
            # Extract the appropriate array for the distribution plot
            if 'avg_values' in data:
                plot_data = data['avg_values']
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

    means = []

    for values in data_to_plot:
        avg = statistics.mean(values)
        means.append(avg)

    fig, ax = plt.subplots(figsize=(max(8, 1.2 * len(labels)), 6))
    x_positions = range(len(labels))

    bars = ax.bar(x_positions, means, alpha=0.85)
    time_labels = [f"{mean:.0f} ns" for mean in means]
    ax.bar_label(bars, labels=time_labels, label_type="center", color="white", fontsize=9)

    min_mean = min(means)
    use_log_scale = min_mean > 0
    if not use_log_scale:
        print("Warning: Non-positive mean encountered; cannot use log scale.")
    else:
        ax.set_yscale("log")

    multipliers = [mean / min_mean for mean in means] if use_log_scale else [None] * len(means)
    for i, (mean, mult) in enumerate(zip(means, multipliers)):
        y_text = mean * 1.08
        text = f"{mult:.2f}x" if mult is not None else "n/a"
        ax.text(i, y_text, text, ha="center", va="bottom")

    ax.set_xticks(list(x_positions))
    ax.set_xticklabels(labels, rotation=30, ha="right")
    ax.set_ylabel("Time (ns)")
    title_suffix = "log scale" if use_log_scale else "linear scale"
    extra = f" - {args.title_extra}" if args.title_extra else ""
    ax.set_title(f"Time per Host Function Call ({title_suffix}){extra}")
    ax.grid(axis="y", linestyle="--", alpha=0.7)

    fig.tight_layout()

    # Save or show
    if args.save:
        plt.savefig(args.save)
        print(f"Plot saved to {args.save}")
    else:
        plt.show()

if __name__ == "__main__":
    main()